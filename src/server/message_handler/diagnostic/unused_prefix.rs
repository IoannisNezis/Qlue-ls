use std::collections::HashSet;

use ll_sparql_parser::{
    ast::{AstNode, PrefixedName, QueryUnit},
    SyntaxNode,
};

use crate::server::lsp::{
    diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
    textdocument::{Range, TextDocumentItem},
};

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    parse_tree: SyntaxNode,
) -> Option<Vec<Diagnostic>> {
    let qu = QueryUnit::cast(parse_tree)?;
    let prefix_declarations = qu.prologue()?.prefix_declarations();
    let used_prefixes: HashSet<String> = qu.select_query().map_or(HashSet::new(), |select_query| {
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
                        code: Some(DiagnosticCode::String("unused-prefix".to_string())),
                        message: format!(
                            "'{}' is declared here, but was never used\n",
                            prefix_declaration.prefix().unwrap_or("prefix".to_string())
                        ),
                        data: None,
                    })
            })
            .collect(),
    )
}
