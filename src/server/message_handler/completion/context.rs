use std::{collections::HashSet, u32};

use ll_sparql_parser::{
    continuations_at, parse_query, syntax_kind::SyntaxKind, SyntaxNode, TokenAtOffset,
};
use text_size::TextSize;

use crate::server::{
    lsp::{errors::ErrorCode, CompletionRequest, CompletionTriggerKind},
    Server,
};

use super::error::CompletionError;

#[derive(Debug)]
pub(super) struct CompletionContext {
    pub(super) location: CompletionLocation,
    pub(super) continuations: HashSet<SyntaxKind>,
    pub(super) trigger_kind: CompletionTriggerKind,
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
        let root = parse_query(&document.text);
        let (location, continuations) = CompletionLocation::from_position(root, offset)?;
        let trigger_kind = request.get_completion_context().trigger_kind.clone();
        Ok(Self {
            location,
            continuations,
            trigger_kind,
        })
    }
}

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
    Object,
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
    SelectBinding,
}

impl CompletionLocation {
    pub(super) fn from_position(
        root: SyntaxNode,
        mut offset: TextSize,
    ) -> Result<(Self, HashSet<SyntaxKind>), CompletionError> {
        let range = root.text_range();

        // NOTE: If the document is empty the cursor is at the beginning
        if range.is_empty() || offset == 0.into() {
            return Ok((CompletionLocation::Start, HashSet::new()));
        }

        if !range.contains(offset) {
            // NOTE: The cursor is "after" the document -> at the end
            if range.end() <= offset {
                offset = root.text_range().end()
            } else {
                log::error!(
                "Requested completion position: ({:?}) before document range ({:?}). This should be impossible.",
                offset,
                range
            );
                return Ok((CompletionLocation::Unknown, HashSet::new()));
            }
        }

        // NOTE: The location of the cursor is not the position we start looking in the tree
        // We start from checking from the first previous non error / non trivia token
        let anchor_token = match root.token_at_offset(offset) {
            TokenAtOffset::Single(mut token) | TokenAtOffset::Between(mut token, _) => {
                // TODO: Handle Comments
                while token.kind() == SyntaxKind::WHITESPACE
                    || token.parent().unwrap().kind() == SyntaxKind::Error
                {
                    if let Some(prev) = token.prev_token() {
                        token = prev
                    } else {
                        return Ok((CompletionLocation::Start, HashSet::new()));
                    }
                }
                token
            }
            TokenAtOffset::None => return Ok((CompletionLocation::Unknown, HashSet::new())),
        };

        Ok(
            if let Some(continuations) = continuations_at(&root, anchor_token.text_range().end()) {
                println!("{:?}", continuations);
                let continuations_set: HashSet<SyntaxKind> =
                    HashSet::from_iter(continuations.into_iter());
                macro_rules! continues_with {
                    ([$($kind:expr),*]) => {
                        [$($kind,)*].iter().any(|kind| continuations_set.contains(kind))
                    };
                }
                // NOTE: Subject
                let location = if continues_with!([
                    SyntaxKind::GroupGraphPatternSub,
                    SyntaxKind::TriplesBlock,
                    SyntaxKind::GraphPatternNotTriples
                ]) {
                    CompletionLocation::Subject
                }
                // FIXME: predicate after ; does not get recognized
                // NOTE: Predicate
                else if continues_with!([
                    SyntaxKind::PropertyListPathNotEmpty,
                    SyntaxKind::PropertyListPath,
                    SyntaxKind::VerbPath,
                    SyntaxKind::VerbSimple
                ]) {
                    CompletionLocation::Predicate
                }
                // NOTE: Object
                else if continues_with!([
                    SyntaxKind::ObjectListPath,
                    SyntaxKind::ObjectPath,
                    SyntaxKind::ObjectList,
                    SyntaxKind::Object
                ]) {
                    CompletionLocation::Object
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
                    && anchor_token
                        .parent_ancestors()
                        .any(|ancestor| ancestor.kind() == SyntaxKind::SelectClause)
                {
                    CompletionLocation::SelectBinding
                } else {
                    CompletionLocation::Unknown
                };
                (location, continuations_set)
            } else {
                // TODO: Can we determin the location even if the
                // continuations are unknown?
                (CompletionLocation::Unknown, HashSet::new())
            },
        )
    }
}
