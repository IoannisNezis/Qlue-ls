//! Flags prefixed names whose prefix is never declared in the prologue.
//!
//! Every `PrefixedName` in the query is checked against the set of declared
//! `PREFIX` labels; any name using an unknown prefix is reported as an error,
//! carrying the prefix so a code action can declare it.

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
    LazyLock::new(|| DiagnosticCode::String("undeclared-prefix".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    ast: &Unit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    let prefixed_names = ast
        .strip_prologue()?
        .descendants()
        .flat_map(&PrefixedName::cast)
        .into_iter();

    let declared_prefixes: HashSet<String> = HashSet::from_iter(
        ast.prologue()
            .map_or(Vec::new(), |prologue| prologue.prefix_declarations())
            .iter()
            .filter_map(|prefix_decl| prefix_decl.prefix()),
    );

    Some(
        prefixed_names
            .into_iter()
            .filter_map(|prefixed_name| {
                (!declared_prefixes.contains(&prefixed_name.prefix())).then(|| Diagnostic {
                    range: Range::from_byte_offset_range(
                        prefixed_name.syntax().text_range(),
                        &document.text,
                    )
                    .expect("prefix declaration text range should be in text"),
                    severity: DiagnosticSeverity::Error,
                    source: Some("qlue-ls".to_string()),
                    code: Some((*CODE).clone()),
                    message: format!(
                        "'{}' is used here, but was never declared\n",
                        prefixed_name.prefix()
                    ),
                    data: Some(LSPAny::String(prefixed_name.prefix())),
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
    fn triggered_by_undeclared_prefix() {
        let result = run(diagnostics, "SELECT * WHERE { ?s schema:name ?o }");
        assert_eq!(result.len(), 1);
        let diagnostic = &result[0];
        assert_eq!(diagnostic.range, Range::new(0, 20, 0, 31));
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.code, Some((*CODE).clone()));
        assert_eq!(diagnostic.data, Some(LSPAny::String("schema".to_string())));
    }

    #[test]
    fn triggered_only_for_undeclared_prefix() {
        let result = run(
            diagnostics,
            indoc!(
                "PREFIX schema: <https://schema.org/>
                 SELECT * WHERE { ?s schema:name ex:o }"
            ),
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].range, Range::new(1, 32, 1, 36));
        assert_eq!(result[0].data, Some(LSPAny::String("ex".to_string())));
    }

    #[test]
    fn not_triggered_by_declared_prefix() {
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
    fn not_triggered_without_prefixed_names() {
        let result = run(
            diagnostics,
            "SELECT * WHERE { ?s <https://schema.org/name> ?o }",
        );
        assert!(result.is_empty());
    }
}
