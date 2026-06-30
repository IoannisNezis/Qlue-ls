//! Flags `PREFIX` declarations in the prologue that declare a prefix label
//! which is already declared earlier in the same prologue.
//!
//! Each redundant declaration is reported so a code action can remove the
//! duplicate.

use crate::{
    Server,
    server::lsp::{
        base_types::LSPAny,
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
};
use ll_sparql_parser::ast::AstNode;
use ll_sparql_parser::ast::QueryUnit;
use std::{collections::HashSet, sync::LazyLock};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("duplicate-prefix-declaration".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    query_unit: &QueryUnit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    let prefix_declarations = query_unit.prologue()?.prefix_declarations();
    let mut seen_prefixes: HashSet<String> = HashSet::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for prefix_decl in prefix_declarations.iter() {
        let prefix = prefix_decl.prefix()?;
        if seen_prefixes.contains(&prefix) {
            diagnostics.push(Diagnostic {
                range: Range::from_byte_offset_range(
                    prefix_decl.syntax().text_range(),
                    &document.text,
                )
                .expect("prefix declaration text range should be in text"),
                severity: DiagnosticSeverity::Warning,
                code: Some((*CODE).clone()),
                source: Some("qlue-ls".to_string()),
                message: format!("The prefix \"{}\" was already declared.\nPrefix declarations should be unique.", prefix),
                data: Some(LSPAny::String(prefix))
            });
        } else {
            seen_prefixes.insert(prefix);
        }
    }
    Some(diagnostics)
}
