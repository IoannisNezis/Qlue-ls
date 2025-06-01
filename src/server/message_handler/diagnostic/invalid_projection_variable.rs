use crate::server::{
    lsp::{
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
    Server,
};
use ll_sparql_parser::ast::{AstNode, QueryUnit};
use std::{collections::HashSet, sync::LazyLock};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("invalid-projection-var".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    query_unit: &QueryUnit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    let projected_variables: Vec<_> = query_unit
        .select_query()?
        .select_clause()?
        .assignments()
        .into_iter()
        .map(|assignment| assignment.variable)
        .collect();
    if projected_variables.is_empty() {
        return None;
    }
    let body_variables: HashSet<String> = query_unit
        .select_query()?
        .where_clause()?
        .visible_variables()
        .into_iter()
        .map(|var| var.text())
        .collect();

    Some(
        projected_variables
            .into_iter()
            .filter_map(|variable| {
                body_variables
                    .contains(&variable.text())
                    .then_some(Diagnostic {
                        code: Some((*CODE).clone()),
                        range: Range::from_byte_offset_range(
                            variable.syntax().text_range(),
                            &document.text,
                        )
                        .unwrap(),
                        severity: DiagnosticSeverity::Error,
                        message: format!(
                            "{} is already defined in the query body",
                            variable.text()
                        ),
                        data: None,
                        source: None,
                    })
            })
            .collect(),
    )
}
