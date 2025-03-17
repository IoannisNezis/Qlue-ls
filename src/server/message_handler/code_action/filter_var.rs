//! Filter variable code action
//! Add Filter expression for variable

use std::collections::HashMap;

use ll_sparql_parser::{
    ast::{AstNode, Var},
    SyntaxToken,
};

use crate::server::lsp::{
    textdocument::{Position, Range, TextDocumentItem, TextEdit},
    CodeAction, WorkspaceEdit,
};

pub(super) fn code_action(token: &SyntaxToken, document: &TextDocumentItem) -> Option<CodeAction> {
    let var = Var::cast(token.parent()?)?;
    let triple = var.triple()?;
    let position =
        Position::from_byte_index(triple.syntax().text_range().end().into(), &document.text)?;
    Some(CodeAction {
        title: "Filter expression".to_string(),
        kind: None,
        diagnostics: vec![],
        edit: WorkspaceEdit {
            changes: HashMap::from_iter([(
                document.uri.to_string(),
                vec![TextEdit::new(
                    Range {
                        start: position,
                        end: position,
                    },
                    &format!(" FILTER ({})", var.syntax()),
                )],
            )]),
        },
    })
}
