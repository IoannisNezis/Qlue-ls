//! Flags variables that are introduced by a projection assignment
//! (`SELECT (... AS ?var)`) while `?var` is already bound in the query body.
//!
//! SPARQL forbids reusing a variable that is visible in the `WHERE` clause as
//! the target of a `SELECT` expression, so each such assignment is reported as
//! an error.

use crate::server::{
    Server,
    lsp::{
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
};
use ll_sparql_parser::ast::{AstNode, Unit};
use std::{collections::HashSet, sync::LazyLock};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("invalid-projection-var".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    ast: &Unit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    let query_unit = ast.as_query()?;
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

#[cfg(test)]
mod test {
    use super::{CODE, diagnostics};
    use crate::server::{
        lsp::{diagnostic::DiagnosticSeverity, textdocument::Range},
        message_handler::diagnostic::test_utils::run,
    };

    #[test]
    fn triggered_by_assignment_to_body_variable() {
        let result = run(diagnostics, "SELECT (?o AS ?s) WHERE { ?s ?p ?o }");
        assert_eq!(result.len(), 1);
        let diagnostic = &result[0];
        assert_eq!(diagnostic.range, Range::new(0, 14, 0, 16));
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.code, Some((*CODE).clone()));
    }

    #[test]
    fn triggered_only_for_bound_variables() {
        let result = run(
            diagnostics,
            "SELECT (?o AS ?x) (?o AS ?p) WHERE { ?s ?p ?o }",
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].range, Range::new(0, 25, 0, 27));
    }

    #[test]
    fn not_triggered_by_fresh_variable() {
        let result = run(diagnostics, "SELECT (?o AS ?x) WHERE { ?s ?p ?o }");
        assert!(result.is_empty());
    }

    #[test]
    fn not_triggered_without_assignments() {
        let result = run(diagnostics, "SELECT ?s WHERE { ?s ?p ?o }");
        assert!(result.is_empty());
    }
}
