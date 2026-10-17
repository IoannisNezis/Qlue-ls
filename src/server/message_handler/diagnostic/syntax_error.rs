//! Reports syntax errors produced by the parser.
//!
//! The document is re-parsed and the first parse error (if any) is surfaced as
//! an error diagnostic at its span. Only the first error is reported to avoid
//! cascading noise from a single mistake.

use std::sync::LazyLock;

use crate::server::{
    Server,
    lsp::{
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
};
use ll_sparql_parser::{ast::Unit, parse};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("syntax-error".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    _ast: &Unit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    let (_, errors) = parse(&document.text);
    (!errors.is_empty()).then_some(
        errors
            .into_iter()
            .take(1)
            .map(|error| Diagnostic {
                range: Range::from_byte_offset_range(error.span, &document.text)
                    .expect("Parse error spans should be within the document"),
                severity: DiagnosticSeverity::Error,
                code: Some((*CODE).clone()),
                source: Some("Qlue-ls".to_string()),
                message: error.message,
                data: None,
            })
            .collect(),
    )
}

#[cfg(test)]
mod test {
    use super::{CODE, diagnostics};
    use crate::server::{
        lsp::diagnostic::DiagnosticSeverity, message_handler::diagnostic::test_utils::run,
    };

    #[test]
    fn triggered_by_invalid_query() {
        let result = run(diagnostics, "SELECT * WHERE { ?s ?p }");
        assert_eq!(result.len(), 1);
        let diagnostic = &result[0];
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.code, Some((*CODE).clone()));
    }

    #[test]
    fn triggered_only_once_for_multiple_errors() {
        let result = run(diagnostics, "SELECT * WHERE { ?s ?p . ?a . ?b }");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn not_triggered_by_valid_query() {
        let result = run(diagnostics, "SELECT * WHERE { ?s ?p ?o }");
        assert!(result.is_empty());
    }

    #[test]
    fn not_triggered_by_valid_update() {
        let result = run(diagnostics, "INSERT DATA { <a> <b> <c> }");
        assert!(result.is_empty());
    }
}
