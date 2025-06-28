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

pub(super) fn code_action(var: &Var, document: &TextDocumentItem) -> Option<CodeAction> {
    let triple = var.triple()?;
    let position =
        Position::from_byte_index(triple.syntax().text_range().end().into(), &document.text)?;
    Some(CodeAction {
        title: "Add Filter".to_string(),
        kind: None,
        diagnostics: vec![],
        edit: WorkspaceEdit {
            changes: Some(HashMap::from_iter([(
                document.uri.to_string(),
                vec![TextEdit::new(
                    Range {
                        start: position,
                        end: position,
                    },
                    &format!(" FILTER ({})", var.syntax()),
                )],
            )])),
        },
    })
}
