//! Filter variable code action
//! Add Lang-Filter expression for variable
//!
//!**Example:**
//!
//! ?s rdfs:label ?label
//!
//! ----------------
//!
//! ?s rdfs:label ?label Filter(Lange(?label) = "en"

use ll_sparql_parser::ast::{AstNode, ObjectList, Var};
use std::collections::HashMap;

use crate::server::lsp::{
    textdocument::{Position, Range, TextDocumentItem, TextEdit},
    CodeAction, WorkspaceEdit,
};

pub(super) fn code_action(var: &Var, document: &TextDocumentItem) -> Option<CodeAction> {
    let triple = var.triple()?;
    let _object_list = var.syntax().ancestors().nth(4).and_then(ObjectList::cast)?;
    let position =
        Position::from_byte_index(triple.syntax().text_range().end().into(), &document.text)?;
    Some(CodeAction {
        title: "Add Lang-Filter".to_string(),
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
                    &format!(r#" FILTER (LANG({}) = "en")"#, var.syntax()),
                )],
            )])),
        },
    })
}
