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
    LazyLock::new(|| DiagnosticCode::String("ungrouped-select-var".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    query_unit: &QueryUnit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    let group_vars_str: HashSet<String> = query_unit
        .select_query()?
        .soulution_modifier()?
        .group_clause()?
        .visible_variables()
        .iter()
        .map(|var| var.text())
        .collect();
    let selected_variables: Vec<_> = query_unit.select_query()?.select_clause()?.variables();
    let unaggregated_variables = query_unit
        .select_query()?
        .select_clause()?
        .assignments()
        .into_iter()
        .flat_map(|assignment| assignment.expression.unaggregated_variables());

    Some(
        selected_variables
            .into_iter()
            .filter_map(|var| {
                (!group_vars_str.contains(&var.text())).then_some(Diagnostic {
                    code: Some((*CODE).clone()),
                    range: Range::from_byte_offset_range(var.syntax().text_range(), &document.text)
                        .unwrap(),
                    severity: DiagnosticSeverity::Error,
                    message: format!("{} is not part of the Group by Clause", var.text()),
                    data: None,
                    source: None,
                })
            })
            .chain(unaggregated_variables.into_iter().filter_map(|var| {
                (!group_vars_str.contains(&var.text())).then_some(Diagnostic {
                    code: Some((*CODE).clone()),
                    range: Range::from_byte_offset_range(var.syntax().text_range(), &document.text)
                        .unwrap(),
                    severity: DiagnosticSeverity::Error,
                    message: format!(
                        "{} is not aggregated or part of the Group by Clause",
                        var.text()
                    ),
                    data: None,
                    source: None,
                })
            }))
            .collect(),
    )
}
