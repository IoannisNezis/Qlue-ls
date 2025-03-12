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
        let location = CompletionLocation::from_position(root, offset)?;
        let trigger_kind = request.get_completion_context().trigger_kind.clone();
        Ok(Self {
            location,
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
    GroupGraphPatternSub,
    /// At a `GroupPatternNotTriples`
    ///
    /// ---
    ///
    /// **Example**
    /// ```sparql
    /// SELECT * WHERE {
    ///   ?a ?b ?c
    ///   >here<
    /// }
    /// ```
    GraphPatternNotTriples,
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
}

impl CompletionLocation {
    pub(super) fn from_position(
        root: SyntaxNode,
        mut offset: TextSize,
    ) -> Result<Self, CompletionError> {
        let range = dbg!(root.text_range());

        // NOTE: If the document is empty the cursor is at the beginning
        if range.is_empty() {
            return Ok(CompletionLocation::Start);
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
                return Ok(CompletionLocation::Unknown);
            }
        }

        // NOTE: The location of the cursor is not the position we start looking in the tree
        // We start from checking from the first previous non error / non trivia token
        let position = match root.token_at_offset(offset) {
            TokenAtOffset::Single(mut token) | TokenAtOffset::Between(mut token, _) => {
                // TODO: Handle Comments
                while token.kind() == SyntaxKind::WHITESPACE
                    || token.parent().unwrap().kind() == SyntaxKind::Error
                {
                    if let Some(prev) = token.prev_token() {
                        token = prev
                    } else {
                        return Ok(CompletionLocation::Start);
                    }
                }
                token.text_range().end()
            }
            TokenAtOffset::None => return Ok(CompletionLocation::Unknown),
        };

        log::info!("Completion position: {:?}", position);

        Ok(
            if let Some(continuations) = continuations_at(&root, position) {
                println!("{:?}", continuations);
                let continuations_set: HashSet<SyntaxKind> =
                    HashSet::from_iter(continuations.into_iter());
                macro_rules! continues_with {
                    ([$($kind:expr),*]) => {
                        [$($kind,)*].iter().any(|kind| continuations_set.contains(kind))
                    };
                }
                // NOTE: GroupGraphPatternSub
                if continues_with!([SyntaxKind::GroupGraphPatternSub, SyntaxKind::TriplesBlock]) {
                    CompletionLocation::GroupGraphPatternSub
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
                else if continues_with!([SyntaxKind::SolutionModifier]) {
                    CompletionLocation::SolutionModifier
                }
                // NOTE: GraphPatternNotTriples
                else if continues_with!([SyntaxKind::GraphPatternNotTriples]) {
                    CompletionLocation::GraphPatternNotTriples
                } else {
                    CompletionLocation::Unknown
                }
            } else {
                // TODO: Can we determin the location even if the
                // continuations are unknown?
                CompletionLocation::Unknown
            },
        )
    }
}
