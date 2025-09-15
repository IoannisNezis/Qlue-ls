mod context;
mod query_graph;
use super::{error::CompletionError, utils::get_prefix_declarations};
use crate::server::{
    lsp::{textdocument::Position, Backend, CompletionRequest, CompletionTriggerKind},
    message_handler::misc::resolve_backend,
    Server,
};
use context::{context, Context};
use futures::lock::Mutex;
use indoc::indoc;
use ll_sparql_parser::{
    ast::{AstNode, BlankPropertyList, QueryUnit, SelectClause, Triple},
    continuations_at, parse,
    syntax_kind::SyntaxKind,
    SyntaxNode, SyntaxToken, TokenAtOffset,
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
    pub(super) backend: Option<Backend>,
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
                 backend:                       {:?}
                "
            },
            self.location,
            self.trigger_textdocument_position,
            self.trigger_character,
            self.continuations,
            self.anchor_token,
            self.search_term,
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
    /// Filter Constraint
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///   FILTER(>here<)
    /// }
    /// ```
    FilterConstraint,
    /// Group Condition
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    /// }
    /// GROUP BY >here<
    /// ```
    GroupCondition,
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
        let offset = document_position
            .position
            .byte_index(&document.text)
            .ok_or(CompletionError::Localization(format!(
                "Position ({}) not inside document range",
                document_position.position
            )))?;
        let trigger_kind = request.get_completion_context().trigger_kind.clone();
        let trigger_character = request.get_completion_context().trigger_character.clone();
        let tree = parse(&document.text);
        let trigger_token = get_trigger_token(&tree, offset);
        let backend = trigger_token.as_ref().and_then(|token| {
            resolve_backend(&server, &QueryUnit::cast(tree.clone())?, &token)
        });
        let anchor_token = trigger_token.and_then(|token| get_anchor_token(token, offset));
        let search_term = get_search_term(&tree, &anchor_token, offset);
        let continuations = get_continuations(&tree, &anchor_token);
        let location = get_location(&anchor_token, &continuations, offset);
        let context = context(&location);
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
        })
    }
}

fn get_search_term(
    root: &SyntaxNode,
    anchor_token: &Option<SyntaxToken>,
    trigger_pos: TextSize,
) -> Option<String> {
    anchor_token.as_ref().and_then(|anchor| {
        let range = if anchor.text_range().end() > trigger_pos {
            TextRange::new(trigger_pos, trigger_pos)
        } else {
            TextRange::new(anchor.text_range().end(), trigger_pos)
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
    })
}

fn get_location(
    anchor_token: &Option<SyntaxToken>,
    continuations: &HashSet<SyntaxKind>,
    offset: TextSize,
) -> CompletionLocation {
    if let Some(anchor) = anchor_token {
        macro_rules! continues_with {
                    ([$($kind:expr),*]) => {
                        [$($kind,)*].iter().any(|kind| continuations.contains(kind))
                    };
                }

        macro_rules! child_of {
                    ([$($kind:expr),*]) => {
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
            if let Some(blank_node_property) =
                anchor.parent_ancestors().find_map(BlankPropertyList::cast)
            {
                CompletionLocation::BlankNodeProperty(blank_node_property)
            } else if let Some(triple) = anchor.parent_ancestors().find_map(Triple::cast) {
                CompletionLocation::Predicate(triple)
            } else {
                CompletionLocation::Unknown
            }
        }
        // NOTE: Subject
        else if continues_with!([
            SyntaxKind::GroupGraphPatternSub,
            SyntaxKind::TriplesBlock,
            SyntaxKind::GraphPatternNotTriples,
            SyntaxKind::DataBlockValue,
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
            if let Some(blank_node_property) =
                anchor.parent_ancestors().find_map(BlankPropertyList::cast)
            {
                CompletionLocation::BlankNodeObject(blank_node_property)
            } else if let Some(triple) = anchor.parent_ancestors().find_map(Triple::cast) {
                CompletionLocation::Object(triple)
            } else {
                CompletionLocation::Unknown
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
            if let Some(select_clause) = anchor
                .parent_ancestors()
                .find(|ancestor| ancestor.kind() == SyntaxKind::SelectClause)
            {
                CompletionLocation::SelectBinding(SelectClause::cast(select_clause).expect(
                    "node of kind SelectClause should be castable to SelectClause ast node",
                ))
            } else {
                CompletionLocation::Unknown
            }
        }
        // NOTE: FilterConstraint
        else if continues_with!([SyntaxKind::Constraint]) && child_of!([SyntaxKind::Filter]) {
            CompletionLocation::FilterConstraint
        }
        // NOTE: GroupCondition
        else if continues_with!([SyntaxKind::GroupCondition]) {
            CompletionLocation::GroupCondition
        } else if continues_with!([SyntaxKind::OrderCondition])
            | (continues_with!([SyntaxKind::BrackettedExpression])
                && child_of!([SyntaxKind::OrderCondition]))
        {
            CompletionLocation::OrderCondition
        } else {
            CompletionLocation::Unknown
        }
    } else {
        CompletionLocation::Start
    }
}

fn get_continuations(root: &SyntaxNode, anchor_token: &Option<SyntaxToken>) -> HashSet<SyntaxKind> {
    if let Some(anchor) = anchor_token.as_ref() {
        if let Some(continuations) = continuations_at(root, anchor.text_range().end()) {
            HashSet::from_iter(continuations)
        } else {
            HashSet::new()
        }
    } else {
        HashSet::new()
    }
}

fn get_trigger_token(root: &SyntaxNode, offset: TextSize) -> Option<SyntaxToken> {
    if offset == 0.into() || root.text_range().end() < offset {
        None
    } else if root.text_range().end() == offset {
        root.last_token()
    } else {
        match root.token_at_offset(offset) {
            TokenAtOffset::Single(token) => Some(token),
            TokenAtOffset::Between(token, _) => Some(token),
            TokenAtOffset::None => None,
        }
    }
}

fn get_anchor_token(
    mut trigger_token: SyntaxToken,
    trigger_offset: TextSize,
) -> Option<SyntaxToken> {
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
    ) &&
        // FIXME: This is also related to THE bug in the tokenizer
        // https://github.com/maciejhirsz/logos/issues/291
        (!matches!(trigger_token.kind(), SyntaxKind::a)
            || trigger_token.text_range().contains(trigger_offset))
    {
        trigger_token = trigger_token.prev_token()?;
    }
    while trigger_token.kind() == SyntaxKind::WHITESPACE
        || trigger_token.parent().unwrap().kind() == SyntaxKind::Error
    {
        if let Some(prev) = trigger_token.prev_token() {
            trigger_token = prev
        } else {
            return None;
        }
    }
    Some(trigger_token)
}

#[cfg(test)]
mod tests;
