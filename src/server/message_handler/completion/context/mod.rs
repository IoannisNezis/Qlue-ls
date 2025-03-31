use std::collections::HashSet;

use ll_sparql_parser::{
    ast::{AstNode, SelectClause, ServiceGraphPattern, Triple},
    continuations_at, parse_query,
    syntax_kind::SyntaxKind,
    SyntaxNode, SyntaxToken, TokenAtOffset,
};
use text_size::{TextRange, TextSize};

use crate::server::{
    lsp::{errors::ErrorCode, textdocument::Position, CompletionRequest, CompletionTriggerKind},
    Server,
};

use super::error::CompletionError;

#[derive(Debug, PartialEq)]
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
}

#[derive(Debug)]
pub(super) struct CompletionContext {
    pub(super) location: CompletionLocation,
    pub(super) trigger_textdocument_position: Position,
    pub(super) continuations: HashSet<SyntaxKind>,
    pub(super) tree: SyntaxNode,
    pub(super) trigger_kind: CompletionTriggerKind,
    pub(super) trigger_character: Option<String>,
    pub(super) anchor_token: Option<SyntaxToken>,
    pub(super) search_term: Option<String>,
    pub(super) backend: Option<String>,
}

impl CompletionContext {
    pub(super) fn from_completion_request(
        server: &Server,
        request: &CompletionRequest,
    ) -> Result<Self, CompletionError> {
        let document_position = request.get_text_position();
        let document = server
            .state
            .get_document(&document_position.text_document.uri)
            .map_err(|err| CompletionError::localization_error(err.code, err.message))?;
        let offset = (document_position
            .position
            .to_byte_index(&document.text)
            .ok_or(CompletionError::localization_error(
                ErrorCode::InvalidParams,
                format!(
                    "Position ({}) not inside document range",
                    document_position.position
                ),
            ))? as u32)
            .into();
        let trigger_kind = request.get_completion_context().trigger_kind.clone();
        let trigger_character = request.get_completion_context().trigger_character.clone();
        let tree = parse_query(&document.text);
        let trigger_token = get_trigger_token(&tree, offset);
        let backend = trigger_token
            .as_ref()
            .unwrap()
            .parent_ancestors()
            .find_map(ServiceGraphPattern::cast)
            .and_then(|service| service.iri())
            .and_then(|iri| iri.raw_iri())
            .and_then(|iri_string| server.state.get_backend_name_by_url(&iri_string))
            .or(server
                .state
                .get_default_backend()
                .map(|backend| backend.name.clone()));
        let anchor_token = trigger_token.and_then(get_anchor_token);
        let search_term = get_search_term(&tree, &anchor_token, offset);
        let continuations = get_continuations(&tree, &anchor_token);
        let location = get_location(&anchor_token, &continuations);
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
            TextRange::new(anchor.text_range().start(), trigger_pos)
        } else {
            TextRange::new(anchor.text_range().end(), trigger_pos)
        };
        root.text_range().contains_range(range).then(|| {
            root.text()
                .slice(range)
                .to_string()
                .trim_start()
                .to_string()
        })
    })
}

fn get_location(
    anchor_token: &Option<SyntaxToken>,
    continuations: &HashSet<SyntaxKind>,
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
            && anchor.parent().map_or(false, |parent| {
                parent.kind() == SyntaxKind::PathOneInPropertySet
            })
        {
            if let Some(triple) = anchor.parent_ancestors().find_map(Triple::cast) {
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
            if let Some(triple) = anchor.parent_ancestors().find_map(Triple::cast) {
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
    if offset == 0.into() || !root.text_range().contains(offset) {
        return None;
    }
    match root.token_at_offset(offset) {
        TokenAtOffset::Single(token) | TokenAtOffset::Between(_, token) => Some(token),
        TokenAtOffset::None => None,
    }
}

fn get_anchor_token(mut token: SyntaxToken) -> Option<SyntaxToken> {
    while token.kind() == SyntaxKind::WHITESPACE
        || token.parent().unwrap().kind() == SyntaxKind::Error
    {
        if let Some(prev) = token.prev_token() {
            token = prev
        } else {
            return None;
        }
    }
    Some(token)
}

#[cfg(test)]
mod tests;
