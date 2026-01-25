use crate::server::{
    Server,
    lsp::{
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
};
use ll_sparql_parser::{ast::QueryUnit, parse};

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    _query_unit: &QueryUnit,
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
                code: Some(DiagnosticCode::String("parse error".to_string())),
                source: Some("Qlue-ls".to_string()),
                message: error.message,
                data: None,
            })
            .collect(),
    )
}
