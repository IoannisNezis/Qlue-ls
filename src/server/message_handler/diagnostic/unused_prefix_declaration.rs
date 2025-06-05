use crate::server::{
    lsp::{
        base_types::LSPAny,
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
    Server,
};
use ll_sparql_parser::ast::{AstNode, PrefixedName, QueryUnit};
use std::{collections::HashSet, sync::LazyLock};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("unused-prefix-declaration".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    query_unit: &QueryUnit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    let prefix_declarations = query_unit.prologue()?.prefix_declarations();
    let used_prefixes: HashSet<String> =
        query_unit
            .select_query()
            .map_or(HashSet::new(), |select_query| {
                HashSet::from_iter(
                    select_query
                        .collect_decendants(&PrefixedName::can_cast)
                        .into_iter()
                        .map(|node| PrefixedName::cast(node).unwrap().prefix()),
                )
            });
    Some(
        prefix_declarations
            .into_iter()
            .filter_map(|prefix_declaration| {
                (!used_prefixes.contains(&prefix_declaration.prefix().unwrap_or("".to_string())))
                    .then(|| Diagnostic {
                        range: Range::from_byte_offset_range(
                            prefix_declaration.syntax().text_range(),
                            &document.text,
                        )
                        .expect("prefix declaration text range should be in text"),
                        severity: DiagnosticSeverity::Warning,
                        source: Some("qlue-ls".to_string()),
                        code: Some((*CODE).clone()),
                        message: format!(
                            "'{}' is declared here, but was never used\n",
                            prefix_declaration.prefix().unwrap_or("prefix".to_string())
                        ),
                        data: prefix_declaration
                            .prefix()
                            .map(|prefix| LSPAny::String(prefix)),
                    })
            })
            .collect(),
    )
}
