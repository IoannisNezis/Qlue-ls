mod context;
mod query_graph;
use super::{error::CompletionError, utils::get_prefix_declarations};
use crate::server::{
    Server,
    configuration::BackendConfiguration,
    lsp::{
        CompletionRequest, CompletionTriggerKind,
        textdocument::{Position, Range},
    },
    message_handler::misc::resolve_backend_at_token,
};
use context::{Context, context};
use futures::lock::Mutex;
use indoc::indoc;
use ll_sparql_parser::{
    SyntaxNode, SyntaxToken, TokenAtOffset,
    ast::{AstNode, BlankPropertyList, InlineData, QueryUnit, SelectClause, Triple},
    continuations_at,
    syntax_kind::SyntaxKind,
};
use std::{collections::HashSet, fmt::Display, rc::Rc, vec};
use text_size::{TextRange, TextSize};

#[derive(Debug, Clone)]
pub(super) struct CompletionEnvironment {
    pub(super) location: CompletionLocation,
    pub(super) trigger_textdocument_position: Position,
    pub(super) continuations: HashSet<SyntaxKind>,
    pub(super) tree: SyntaxNode,
    pub(super) trigger_kind: CompletionTriggerKind,
    pub(super) trigger_character: Option<String>,
    pub(super) anchor_token: Option<SyntaxToken>,
    pub(super) search_term: Option<String>,
    pub(super) replace_range: Range,
    pub(super) backend: Option<BackendConfiguration>,
    pub(super) context: Option<Context>,
}

impl Display for CompletionEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            indoc! {
                "location:                      {:?}
                 trigger_textdocument_position: {}
                 trigger_kind:                  {:?}
                 continuations:                 {:?}
                 anchor_token:                  {:?}
                 search_term:                   {:?}
                 replace_range:                 {}
                 backend:                       {:?}
                "
            },
            self.location,
            self.trigger_textdocument_position,
            self.trigger_character,
            self.continuations,
            self.anchor_token,
            self.search_term,
            self.replace_range,
            self.backend.as_ref().map(|backend| &backend.name)
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(super) enum CompletionLocation {
    /// Unsupported location
    Unknown,
    /// At the beginning of the input
    Start,
    /// Inside a "{}" Block
    /// Either at a `TriplesBlock` or a `GroupPatternNotTriples`
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///  >here<
    /// }
    /// ```
    /// or
    /// ```sparql
    /// SELECT * WHERE {
    ///   OPTIONAL {
    ///     ?s ?p ?o .
    ///     >here<
    ///   }
    /// }
    /// ```
    Subject,
    /// 2nd part of a Triple
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///  ?subject >here<
    /// }
    /// ```
    /// or
    /// ```sparql
    /// SELECT * WHERE {
    ///  ?s ?p ?o ;
    ///     >here<
    /// }
    /// ```
    Predicate(Triple),
    /// 3rd part of a Triple
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///  ?subject <someiri> >here<
    /// }
    /// ```
    Object(Triple),
    /// After a Select Query
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///  ?s ?p ?o
    /// }
    /// >here<
    /// ```
    /// or
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///  ?s ?p ?o
    /// }
    /// GROUP By ?s
    /// >here<
    SolutionModifier,
    /// Variable Or Assignment in SelectClause
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT >here< ?s >here< WHERE {}
    /// ```
    /// or
    /// ```sparql
    /// SELECT REDUCED >here< WHERE {}
    /// ```
    SelectBinding(SelectClause),
    /// RDF Graph identifier
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///   Graph >here< {
    ///   }
    /// }
    /// ```
    Graph,
    /// A Blank node property list in a triple
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///   ?s ?p [ >here< ]
    /// }
    ///
    /// or
    ///
    /// ```sparql
    /// SELECT * WHERE {
    ///   \[ >here< \]
    /// }
    /// ```
    BlankNodeProperty(BlankPropertyList),
    /// A Blank node object list in a triple
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///   ?s ?p [ ?p2 >here< ]
    /// }
    ///
    /// or
    ///
    /// ```sparql
    /// SELECT * WHERE {
    ///   \[ ?p >here< \]
    /// }
    /// ```
    BlankNodeObject(BlankPropertyList),
    /// URL of a SERVICE endpoint
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///   SERVICE >here< {}
    /// }
    ServiceUrl,
    /// Order Condition
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    /// }
    /// ORDER BY >here<
    /// ```
    OrderCondition,

    /// Inline Data (aka VALUES clause)
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///   VALUES ?x {>here<}
    /// }
    /// ```
    InlineData((InlineData, usize)),

    /// BuiltInCall
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///   FILTER(>here<)
    /// }
    /// ```
    BuiltInCall,
}

impl CompletionEnvironment {
    /// Create a tera template context filled with the following variables:
    ///
    /// - `search_term` : query string to find the entity
    /// - `context` : connected triples for context sensitive completions
    /// - `prefixes` : used prefixes in this query
    pub(super) async fn template_context(&self) -> tera::Context {
        let mut template_context = tera::Context::new();
        template_context.insert("context", &self.context);
        let mut prefixes = match &self.location {
            CompletionLocation::Predicate(triple) | CompletionLocation::Object(triple) => {
                triple.used_prefixes()
            }
            CompletionLocation::BlankNodeObject(blank_property_list) => {
                blank_property_list.used_prefixes()
            }
            CompletionLocation::SelectBinding(select_clause) => select_clause.used_prefixes(),
            CompletionLocation::BlankNodeProperty(blank_property_list) => {
                blank_property_list.used_prefixes()
            }
            _ => vec![],
        };
        if let Some(ref context) = self.context {
            prefixes.extend(context.prefixes.clone());
        }
        let prefix_declarations = get_prefix_declarations(&self.tree).await;
        template_context.insert("prefixes", &prefix_declarations);
        template_context.insert("search_term", &self.search_term);
        if let Some(search_term) = self.search_term.as_ref() {
            // NOTE: if the search term contains a ":"
            // its very likely the user is typing a prefix
            // to handle this: decompress the prefix and
            // augment the search_term
            if let Some(uncompressed) = search_term.find(":").map(|idx| {
                let (prefix, resource) = search_term.split_at(idx);
                prefix_declarations
                    .iter()
                    .find_map(|prefix_decl| {
                        (prefix_decl.0 == prefix).then_some(prefix_decl.1.clone())
                    })
                    .map(|uri_prefix| uri_prefix + &resource[1..])
            }) {
                template_context.insert("search_term_uncompressed", &uncompressed);
            }
        }

        template_context
    }

    pub(super) async fn from_completion_request(
        server_rc: Rc<Mutex<Server>>,
        request: &CompletionRequest,
    ) -> Result<Self, CompletionError> {
        let server = server_rc.lock().await;
        let document_position = request.get_text_position();
        let document = server
            .state
            .get_document(&document_position.text_document.uri)
            .map_err(|err| CompletionError::Localization(err.message))?;
        let trigger_offset = document_position
            .position
            .byte_index(&document.text)
            .ok_or(CompletionError::Localization(format!(
                "Position ({}) not inside document range",
                document_position.position
            )))?;
        let trigger_kind = request.get_completion_context().trigger_kind.clone();
        let trigger_character = request.get_completion_context().trigger_character.clone();

        let tree = server
            .state
            .get_cached_parse_tree(&document_position.text_document.uri)
            .map_err(|err| CompletionError::Localization(err.message))?
            .tree;
        let trigger_token = get_trigger_token(&tree, trigger_offset);
        let backend = trigger_token
            .as_ref()
            .and_then(|token| {
                resolve_backend_at_token(&server, &QueryUnit::cast(tree.clone())?, token)
            })
            .or_else(|| server.state.get_default_backend().cloned());
        let anchor_token = trigger_token.and_then(get_anchor_token);
        let search_term = get_search_term(&tree, &anchor_token, trigger_offset);
        let continuations = get_continuations(&tree, &anchor_token);
        let location = get_location(&anchor_token, &continuations, trigger_offset);
        let context = context(&location);
        let replace_range = get_replace_range(&document_position.position, &search_term);
        Ok(Self {
            location,
            trigger_textdocument_position: document_position.position,
            continuations,
            tree,
            trigger_kind,
            trigger_character,
            anchor_token,
            search_term,
            backend,
            context,
            replace_range,
        })
    }
}

fn get_search_term(
    root: &SyntaxNode,
    anchor_token: &Option<SyntaxToken>,
    trigger_pos: TextSize,
) -> Option<String> {
    let range = if let Some(anchor_token) = anchor_token {
        if anchor_token.text_range().end() > trigger_pos {
            TextRange::new(trigger_pos, trigger_pos)
        } else {
            TextRange::new(anchor_token.text_range().end(), trigger_pos)
        }
    } else {
        // NOTE: if the anchor token is None:
        // - either the trigger_token was None -> triggered at start
        // - or there is no non-trivia token before (including) the trigger token
        // That means the completion was triggered at the start
        TextRange::new(0.into(), trigger_pos)
    };

    root.text_range()
        .contains_range(range)
        .then_some({
            root.text()
                .slice(range)
                .to_string()
                .trim_start()
                .to_string()
        })
        .filter(|search_term|
                // NOTE: If the search term contains just is_whitespace
                search_term.chars().any(|char| !char.is_whitespace()))
}

fn get_location(
    anchor_token: &Option<SyntaxToken>,
    continuations: &HashSet<SyntaxKind>,
    offset: TextSize,
) -> CompletionLocation {
    if let Some(anchor) = anchor_token {
        macro_rules! continues_with {
                    ([$($kind:expr_2021),*]) => {
                        [$($kind,)*].iter().any(|kind| continuations.contains(kind))
                    };
                }

        macro_rules! child_of {
                    ([$($kind:expr_2021),*]) => {
                        [$($kind,)*].iter().any(|kind| anchor.parent().map_or(false, |parent| parent.kind() == *kind))
                    };
                }
        // NOTE: START
        if anchor.kind() == SyntaxKind::WHITESPACE && anchor.text_range().start() == 0.into() {
            CompletionLocation::Start
        } else if anchor.kind() == SyntaxKind::ANON && anchor.text_range().contains(offset) {
            anchor
                .parent()
                .and_then(|parent| {
                    BlankPropertyList::cast(parent).map(CompletionLocation::BlankNodeProperty)
                })
                .unwrap_or(CompletionLocation::Unknown)
        }
        // NOTE: Predicate
        else if continues_with!([
            SyntaxKind::PropertyListPathNotEmpty,
            SyntaxKind::PropertyListPath,
            SyntaxKind::Path,
            SyntaxKind::VerbPath,
            SyntaxKind::VerbSimple,
            SyntaxKind::PathEltOrInverse,
            SyntaxKind::PathSequence,
            SyntaxKind::PathElt,
            SyntaxKind::PathNegatedPropertySet,
            SyntaxKind::PathOneInPropertySet,
            SyntaxKind::PathAlternative
        ]) || continues_with!([SyntaxKind::iri])
            && anchor
                .parent()
                .is_some_and(|parent| parent.kind() == SyntaxKind::PathOneInPropertySet)
        {
            match anchor.parent_ancestors().find_map(BlankPropertyList::cast) {
                Some(blank_node_property) => {
                    CompletionLocation::BlankNodeProperty(blank_node_property)
                }
                _ => match anchor.parent_ancestors().find_map(Triple::cast) {
                    Some(triple) => CompletionLocation::Predicate(triple),
                    _ => CompletionLocation::Unknown,
                },
            }
        }
        // NOTE: Subject
        else if continues_with!([
            SyntaxKind::GroupGraphPatternSub,
            SyntaxKind::TriplesBlock,
            SyntaxKind::GraphPatternNotTriples,
            SyntaxKind::GraphNodePath
        ]) {
            CompletionLocation::Subject
        }
        // NOTE: Object
        else if continues_with!([
            SyntaxKind::ObjectListPath,
            SyntaxKind::ObjectPath,
            SyntaxKind::ObjectList,
            SyntaxKind::Object
        ]) {
            match anchor.parent_ancestors().find_map(BlankPropertyList::cast) {
                Some(blank_node_property) => {
                    CompletionLocation::BlankNodeObject(blank_node_property)
                }
                _ => match anchor.parent_ancestors().find_map(Triple::cast) {
                    Some(triple) => CompletionLocation::Object(triple),
                    _ => CompletionLocation::Unknown,
                },
            }
        }
        // NOTE: SolutionModifier
        else if continues_with!([
            SyntaxKind::SolutionModifier,
            SyntaxKind::HavingClause,
            SyntaxKind::OrderClause,
            SyntaxKind::LimitOffsetClauses,
            SyntaxKind::LimitClause,
            SyntaxKind::OffsetClause
        ]) {
            CompletionLocation::SolutionModifier
        } else if (continues_with!([SyntaxKind::VarOrIri])
            && child_of!([SyntaxKind::GraphGraphPattern]))
            || continues_with!([SyntaxKind::DefaultGraphClause, SyntaxKind::SourceSelector])
        {
            CompletionLocation::Graph
        }
        // NOTE: ServiceUrl
        else if continues_with!([SyntaxKind::VarOrIri])
            && child_of!([SyntaxKind::ServiceGraphPattern])
        {
            CompletionLocation::ServiceUrl
        }
        // NOTE: SelectBinding
        else if continues_with!([SyntaxKind::Var])
            && anchor
                .parent_ancestors()
                .any(|ancestor| ancestor.kind() == SyntaxKind::SelectClause)
        {
            match anchor
                .parent_ancestors()
                .find(|ancestor| ancestor.kind() == SyntaxKind::SelectClause)
            {
                Some(select_clause) => {
                    CompletionLocation::SelectBinding(SelectClause::cast(select_clause).expect(
                        "node of kind SelectClause should be castable to SelectClause ast node",
                    ))
                }
                _ => CompletionLocation::Unknown,
            }
        } else if continues_with!([SyntaxKind::OrderCondition])
            | (continues_with!([SyntaxKind::BrackettedExpression])
                && child_of!([SyntaxKind::OrderCondition]))
        {
            CompletionLocation::OrderCondition
        }
        // NOTE: InlineData
        else if continues_with!([SyntaxKind::DataBlockValue]) {
            match anchor.parent_ancestors().find_map(InlineData::cast) {
                Some(inline_data) => {
                    let index = inline_data_variable_index(&inline_data, offset);
                    CompletionLocation::InlineData((inline_data, index))
                }
                None => CompletionLocation::Unknown,
            }
        }
        //NOTE: BuiltInCall
        else if continues_with!([
            SyntaxKind::Constraint,
            SyntaxKind::Expression,
            SyntaxKind::GroupCondition
        ]) {
            CompletionLocation::BuiltInCall
        } else {
            CompletionLocation::Unknown
        }
    } else {
        CompletionLocation::Start
    }
}

/// Compute which variable position the cursor is at inside an InlineData (VALUES) clause.
/// For `VALUES (?x ?y) { (<a> |) }` with cursor at `|`, returns 1 (the ?y slot).
fn inline_data_variable_index(inline_data: &InlineData, offset: TextSize) -> usize {
    // NOTE: InlineData → DataBlock → InlineDataOneVar | InlineDataFull
    let Some(data_block) = inline_data.syntax().last_child() else {
        return 0;
    };
    let Some(inner) = data_block.first_child() else {
        return 0;
    };
    if inner.kind() != SyntaxKind::InlineDataFull {
        return 0;
    }
    let mut past_lcurly = false;
    let mut counter = 0;
    for child in inner.children_with_tokens() {
        if child.kind() == SyntaxKind::LCurly {
            past_lcurly = true;
            continue;
        }
        if !past_lcurly {
            continue;
        }
        if child.kind() == SyntaxKind::LParen {
            counter = 0;
        } else if child.kind() == SyntaxKind::DataBlockValue && child.text_range().end() <= offset {
            counter += 1;
        }
        if child.text_range().start() >= offset {
            return counter;
        }
    }
    counter
}

fn get_continuations(root: &SyntaxNode, anchor_token: &Option<SyntaxToken>) -> HashSet<SyntaxKind> {
    // NOTE: The anchor token might be a comment token.
    // I this case: search for the previous non-trivia token.
    let mut start_token = anchor_token.clone();
    loop {
        if let Some(token) = start_token.as_ref() {
            if token.kind().is_trivia() {
                start_token = token.prev_token();
            } else {
                break;
            }
        } else {
            return HashSet::new();
        }
    }
    if let Some(continuations) = continuations_at(
        root,
        start_token
            .expect("Start token should exist")
            .text_range()
            .end(),
    ) {
        HashSet::from_iter(continuations)
    } else {
        HashSet::new()
    }
}

/// Get the last token before the trigger offset.
/// There are 3 options.
/// - the trigger offset is infront of any non-trivia token -> return None
/// - the trigger offset is within a token span -> return this token
/// - the trigger offset is between two tokens -> return left token
/// - the trigger offset is not withing the parse tree span -> fallback to last token (or None).
///
/// **Important:** if the result is None, the completion is triggered at the start
fn get_trigger_token(root: &SyntaxNode, offset: TextSize) -> Option<SyntaxToken> {
    if offset == 0.into() {
        None
    } else if root.text_range().end() <= offset {
        // last_token() can return None if the tree ends with an empty node (e.g., Error@27..27).
        // In that case, fall back to iterating through all tokens.
        root.last_token().or_else(|| {
            root.descendants_with_tokens()
                .filter_map(|it| it.into_token())
                .last()
        })
    } else {
        match root.token_at_offset(offset) {
            TokenAtOffset::Single(token) => Some(token),
            TokenAtOffset::Between(token, _) => Some(token),
            TokenAtOffset::None => None,
        }
    }
}

fn get_anchor_token(mut trigger_token: SyntaxToken) -> Option<SyntaxToken> {
    // NOTE: Skip first token in some cases:
    if !matches!(
        trigger_token.kind(),
        SyntaxKind::Error
            | SyntaxKind::WHITESPACE
            | SyntaxKind::Dot
            | SyntaxKind::Semicolon
            | SyntaxKind::RBrack
            | SyntaxKind::RCurly
            | SyntaxKind::RParen
            | SyntaxKind::Slash
            | SyntaxKind::Zirkumflex
            | SyntaxKind::ANON
            | SyntaxKind::LParen
    ) {
        trigger_token = trigger_token.prev_token()?;
    }
    while trigger_token.kind() == SyntaxKind::WHITESPACE
        || trigger_token.parent().unwrap().kind() == SyntaxKind::Error
    {
        match trigger_token.prev_token() {
            Some(prev) => trigger_token = prev,
            _ => {
                return None;
            }
        }
    }
    Some(trigger_token)
}

/// Get the range the completion is supposed to replace
fn get_replace_range(trigger_pos: &Position, search_term: &Option<String>) -> Range {
    Range {
        start: Position::new(
            trigger_pos.line,
            trigger_pos.character
                - search_term
                    .as_ref()
                    .map(|search_term| {
                        search_term
                            .chars()
                            .map(|char| char.len_utf16())
                            .sum::<usize>() as u32
                    })
                    .unwrap_or(0),
        ),
        end: *trigger_pos,
    }
}

#[cfg(test)]
mod tests;
