//! Flags `PREFIX` declarations in the prologue whose prefix is never used by
//! any prefixed name in the rest of the query.
//!
//! Each unused declaration is reported as a warning, carrying the prefix so a
//! code action can remove the declaration.

use crate::server::{
    Server,
    lsp::{
        base_types::LSPAny,
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
};
use ll_sparql_parser::ast::{AstNode, PrefixedName, Unit};
use std::{collections::HashSet, sync::LazyLock};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("unused-prefix-declaration".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    ast: &Unit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    let prefix_declarations = ast.prologue()?.prefix_declarations();
    let used_prefixes: HashSet<String> = ast.strip_prologue().map_or(HashSet::new(), |query| {
        HashSet::from_iter(
            query
                .descendants()
                .flat_map(&PrefixedName::cast)
                .map(|node| node.prefix()),
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
                        data: prefix_declaration.prefix().map(LSPAny::String),
                    })
            })
            .collect(),
    )
}

#[cfg(test)]
mod test {
    use super::{CODE, diagnostics};
    use crate::server::{
        lsp::{base_types::LSPAny, diagnostic::DiagnosticSeverity, textdocument::Range},
        message_handler::diagnostic::test_utils::run,
    };
    use indoc::indoc;

    #[test]
    fn triggered_by_unused_prefix() {
        let result = run(
            diagnostics,
            indoc!(
                "PREFIX schema: <https://schema.org/>
                 SELECT * WHERE { ?s ?p ?o }"
            ),
        );
        assert_eq!(result.len(), 1);
        let diagnostic = &result[0];
        assert_eq!(diagnostic.range, Range::new(0, 0, 0, 36));
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostic.code, Some((*CODE).clone()));
        assert_eq!(diagnostic.data, Some(LSPAny::String("schema".to_string())));
    }

    #[test]
    fn triggered_only_for_unused_prefix() {
        let result = run(
            diagnostics,
            indoc!(
                "PREFIX schema: <https://schema.org/>
                 PREFIX ex: <http://example.org/>
                 SELECT * WHERE { ?s schema:name ?o }"
            ),
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].range, Range::new(1, 0, 1, 32));
        assert_eq!(result[0].data, Some(LSPAny::String("ex".to_string())));
    }

    #[test]
    fn not_triggered_by_used_prefix() {
        let result = run(
            diagnostics,
            indoc!(
                "PREFIX schema: <https://schema.org/>
                 SELECT * WHERE { ?s schema:name ?o }"
            ),
        );
        assert!(result.is_empty());
    }

    #[test]
    fn not_triggered_without_prologue() {
        let result = run(diagnostics, "SELECT * WHERE { ?s ?p ?o }");
        assert!(result.is_empty());
    }
}
