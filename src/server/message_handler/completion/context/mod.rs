use std::collections::HashSet;

use ll_sparql_parser::{
    ast::{AstNode, SelectClause, Triple},
    continuations_at, parse_query,
    syntax_kind::SyntaxKind,
    SyntaxNode, SyntaxToken, TokenAtOffset,
};
use text_size::TextSize;

use crate::server::{
    lsp::{errors::ErrorCode, CompletionRequest, CompletionTriggerKind},
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
    Predicate,
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
}

#[derive(Debug)]
pub(super) struct CompletionContext {
    pub(super) location: CompletionLocation,
    pub(super) continuations: HashSet<SyntaxKind>,
    pub(super) tree: SyntaxNode,
    pub(super) trigger_kind: CompletionTriggerKind,
    pub(super) trigger_character: Option<String>,
    pub(super) anchor_token: Option<SyntaxToken>,
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
        let anchor_token = get_anchor_token(&tree, offset);
        log::info!("anchor: {:?}", anchor_token);
        let continuations = get_continuations(&tree, &anchor_token);
        log::info!("continuations: {:?}", continuations);
        let location = get_location(&anchor_token, &continuations);
        log::info!("location: {:?}", location);
        Ok(Self {
            location,
            continuations,
            tree,
            trigger_kind,
            trigger_character,
            anchor_token,
        })
    }
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
        // NOTE: START
        if anchor.kind() == SyntaxKind::WHITESPACE && anchor.text_range().start() == 0.into() {
            CompletionLocation::Start
        }
        // NOTE: Predicate
        else if continues_with!([
            SyntaxKind::PropertyListPathNotEmpty,
            SyntaxKind::PropertyListPath,
            SyntaxKind::VerbPath,
            SyntaxKind::VerbSimple
        ]) {
            CompletionLocation::Predicate
        }
        // NOTE: Subject
        else if continues_with!([
            SyntaxKind::GroupGraphPatternSub,
            SyntaxKind::TriplesBlock,
            SyntaxKind::GraphPatternNotTriples,
            SyntaxKind::DataBlockValue
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
            if let Some(triple) = anchor
                .parent_ancestors()
                .find(|parent| Triple::can_cast(parent.kind()))
            {
                CompletionLocation::Object(Triple::cast(triple).unwrap())
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

fn get_anchor_token(root: &SyntaxNode, offset: TextSize) -> Option<SyntaxToken> {
    if offset == 0.into() {
        return None;
    }
    match root.token_at_offset(offset) {
        TokenAtOffset::Single(mut token) | TokenAtOffset::Between(mut token, _) => {
            // TODO: Handle Comments
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
        TokenAtOffset::None => None,
    }
}

#[cfg(test)]
mod tests;
