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
use ll_sparql_parser::ast::{AstNode, Unit};
use std::{collections::HashSet, sync::LazyLock};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("duplicate-prefix-declaration".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    ast: &Unit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    let prefix_declarations = ast.prologue()?.prefix_declarations();
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

#[cfg(test)]
mod test {
    use super::{CODE, diagnostics};
    use crate::server::{
        lsp::{base_types::LSPAny, diagnostic::DiagnosticSeverity, textdocument::Range},
        message_handler::diagnostic::test_utils::run,
    };
    use indoc::indoc;

    #[test]
    fn triggered_by_duplicate_prefix() {
        let result = run(
            diagnostics,
            indoc!(
                "PREFIX a: <http://a.org/>
                 PREFIX a: <http://b.org/>
                 SELECT * WHERE { ?s a:p a:o }"
            ),
        );
        assert_eq!(result.len(), 1);
        let diagnostic = &result[0];
        assert_eq!(diagnostic.range, Range::new(1, 0, 1, 25));
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostic.code, Some((*CODE).clone()));
        assert_eq!(diagnostic.data, Some(LSPAny::String("a".to_string())));
    }

    #[test]
    fn triggered_once_per_redundant_declaration() {
        let result = run(
            diagnostics,
            indoc!(
                "PREFIX a: <http://a.org/>
                 PREFIX b: <http://b.org/>
                 PREFIX a: <http://a.org/>
                 PREFIX a: <http://c.org/>
                 SELECT * WHERE { ?s a:p b:o }"
            ),
        );
        assert_eq!(
            result.iter().map(|d| d.range.clone()).collect::<Vec<_>>(),
            vec![Range::new(2, 0, 2, 25), Range::new(3, 0, 3, 25)]
        );
    }

    #[test]
    fn not_triggered_by_unique_prefixes() {
        let result = run(
            diagnostics,
            indoc!(
                "PREFIX a: <http://a.org/>
                 PREFIX b: <http://a.org/>
                 SELECT * WHERE { ?s a:p b:o }"
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
