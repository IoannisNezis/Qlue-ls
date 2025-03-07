use std::u32;

use ll_sparql_parser::{
    parse_query, syntax_kind::SyntaxKind, SyntaxNode, SyntaxToken, TokenAtOffset,
};
use text_size::TextSize;

use crate::server::{
    lsp::{errors::ErrorCode, CompletionRequest, CompletionTriggerKind},
    message_handler::completion::utils::match_ancestors,
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
    /// At the beginning of the input
    End,
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
    TripleOrNotTriple,
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
}

impl CompletionLocation {
    pub(super) fn from_token(token: &SyntaxToken) -> Self {
        if let Some(location) = match token
            .prev_sibling_or_token()
            .map_or(SyntaxKind::Eof, |prev| prev.kind())
        {
            SyntaxKind::VarOrTerm => Some(CompletionLocation::Predicate),
            SyntaxKind::VerbPath | SyntaxKind::VerbSimple => Some(CompletionLocation::Object),
            SyntaxKind::Query => Some(CompletionLocation::End),
            _ => None,
        } {
            return location;
        }
        if let Some(location) = match token
            .parent()
            .map_or(SyntaxKind::Eof, |parent| parent.kind())
        {
            SyntaxKind::GroupGraphPattern | SyntaxKind::TriplesBlock => {
                Some(CompletionLocation::TripleOrNotTriple)
            }
            SyntaxKind::QueryUnit => Some(CompletionLocation::Start),
            _ => None,
        } {
            return location;
        }
        if match_ancestors(&token, &[SyntaxKind::Error, SyntaxKind::GroupGraphPattern]) {
            return CompletionLocation::TripleOrNotTriple;
        }
        if match_ancestors(&token, &[SyntaxKind::Error, SyntaxKind::Query]) {
            return CompletionLocation::Start;
        }
        CompletionLocation::Unknown
    }
    pub(super) fn from_position(
        root: SyntaxNode,
        offset: TextSize,
    ) -> Result<Self, CompletionError> {
        let range = root.text_range();
        if range.is_empty() {
            return Ok(CompletionLocation::Start);
        }
        if !range.contains(offset) {
            if range.end() <= offset {
                return Ok(CompletionLocation::End);
            }
            log::error!(
                "Requested completion position: ({:?}) before document range ({:?}). This should be impossible.",
                offset,
                range
            );
            return Ok(CompletionLocation::Unknown);
        }

        Ok(match root.token_at_offset(offset) {
            TokenAtOffset::Single(token) => {
                if token.kind() == SyntaxKind::WHITESPACE {
                    CompletionLocation::from_token(&token)
                } else {
                    CompletionLocation::Unknown
                }
            }
            TokenAtOffset::Between(token1, _token2) => CompletionLocation::from_token(&token1),
            TokenAtOffset::None => CompletionLocation::Start,
        })
    }
}
