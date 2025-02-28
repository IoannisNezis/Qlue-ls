use std::u32;

use ll_sparql_parser::{
    parse_query, print_full_tree, syntax_kind::SyntaxKind, SyntaxNode, SyntaxToken, TokenAtOffset,
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
    Unknown,
    /// In empty File
    Empty,
    /// Inside a "{}" Block
    /// Either at a `TriplesBlock` or a `GroupPatternNotTriples`
    /// ```sparql
    /// SELECT * WHERE {
    ///  <here>
    /// }
    /// ```
    /// or
    /// ```sparql
    /// SELECT * WHERE {
    ///   OPTIONAL {
    ///     ?s ?p ?o .
    ///     <here>
    ///   }
    /// }
    /// ```
    TripleOrNotTriple,
    /// 2nd part of a Triple
    /// ```sparql
    /// SELECT * WHERE {
    ///  ?subject <here>
    /// }
    /// ```
    Predicate,
    Object,
    SolutionModifier,
}

impl CompletionLocation {
    pub(super) fn from_position(
        root: SyntaxNode,
        offset: TextSize,
    ) -> Result<Self, CompletionError> {
        log::info!("{}", print_full_tree(&root, 0));
        let range = root.text_range();
        if range.is_empty() {
            return Ok(CompletionLocation::Empty);
        }
        if !range.contains(offset) {
            if range.end() < offset {
                return Ok(CompletionLocation::SolutionModifier);
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
                    match token
                        .prev_sibling_or_token()
                        .map_or(SyntaxKind::Eof, |prev| prev.kind())
                    {
                        SyntaxKind::VarOrTerm => CompletionLocation::Predicate,
                        SyntaxKind::VerbPath | SyntaxKind::VerbSimple => CompletionLocation::Object,
                        _ => match token
                            .parent()
                            .map_or(SyntaxKind::Eof, |parent| parent.kind())
                        {
                            SyntaxKind::GroupGraphPattern | SyntaxKind::TriplesBlock => {
                                CompletionLocation::TripleOrNotTriple
                            }
                            _ => CompletionLocation::Unknown,
                        },
                    }
                } else {
                    CompletionLocation::Unknown
                }
            }
            TokenAtOffset::Between(token1, _token2) => {
                token1
                    .parent_ancestors()
                    .for_each(|node| log::info!("{:?}", node.kind()));
                if match_ancestors(&token1, &[SyntaxKind::Error, SyntaxKind::Query]) {
                    CompletionLocation::Empty
                } else {
                    CompletionLocation::Unknown
                }
            }
            TokenAtOffset::None => {
                log::info!("at no token");
                CompletionLocation::Empty
            }
        })
    }
}

fn match_ancestors(token: &SyntaxToken, ancestors: &[SyntaxKind]) -> bool {
    token
        .parent_ancestors()
        .zip(ancestors.iter())
        .take_while(|(ancestor, kind)| ancestor.kind() == **kind)
        .count()
        == ancestors.len()
}
