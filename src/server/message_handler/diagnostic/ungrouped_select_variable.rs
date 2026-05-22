use crate::server::{
    Server,
    lsp::{
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
};
use ll_sparql_parser::ast::{AstNode, QueryUnit};
use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("ungrouped-select-var".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    query_unit: &QueryUnit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    // NOTE: The variables specified in the GROUP BY
    let group_vars_str: HashSet<String> = query_unit
        .select_query()?
        .soulution_modifier()?
        .group_clause()?
        .visible_variables()
        .iter()
        .map(|var| var.text())
        .collect();

    // NOTE: ALL variables used SELECT CLAUSE
    let selected_variables: Vec<_> = query_unit.select_query()?.select_clause()?.variables();

    // NOTE: ALL variables used in assignments in the SELECT CLAUSE that are not part of an aggregation
    let unaggregated_variables = query_unit
        .select_query()?
        .select_clause()?
        .assignments()
        .into_iter()
        .enumerate()
        .flat_map(|(idx, assignment)| {
            assignment
                .expression
                .unaggregated_variables()
                .into_iter()
                .map(move |var| (idx, var))
        });

    // NOTE: The variables derived in assignments in the SELECT CLAUSE
    let derived_variables: HashMap<String, usize> = HashMap::from_iter(
        query_unit
            .select_query()?
            .select_clause()?
            .assignments()
            .into_iter()
            .enumerate()
            .map(|(idx, assignment)| (assignment.variable.text(), idx)),
    );

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
            .chain(
                unaggregated_variables
                    .into_iter()
                    .filter_map(|(assignment_idx, var)| {
                        (!group_vars_str.contains(&var.text())
                            && derived_variables
                                .get(&var.text())
                                .is_none_or(|derived_idx| *derived_idx > assignment_idx))
                        .then_some(Diagnostic {
                            code: Some((*CODE).clone()),
                            range: Range::from_byte_offset_range(
                                var.syntax().text_range(),
                                &document.text,
                            )
                            .unwrap(),
                            severity: DiagnosticSeverity::Error,
                            message: format!(
                                "{} is not aggregated or part of the Group by Clause",
                                var.text()
                            ),
                            data: None,
                            source: None,
                        })
                    }),
            )
            .collect(),
    )
}
