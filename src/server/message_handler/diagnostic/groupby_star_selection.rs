use std::sync::LazyLock;

use crate::{
    Server,
    server::lsp::{
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
};
use ll_sparql_parser::ast::{AstNode, SelectQuery, Unit};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("groupby-star-selection".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    ast: &Unit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    let query_unit = ast.as_query()?;
    let select_queries: Vec<SelectQuery> = query_unit
        .syntax()
        .descendants()
        .filter_map(SelectQuery::cast)
        .collect();
    let diagnostics: Vec<_> = select_queries
        .into_iter()
        .filter_map(|select_query| invalid_selection(&select_query, document))
        .collect();
    (!diagnostics.is_empty()).then_some(diagnostics)
}

fn invalid_selection(
    select_query: &SelectQuery,
    document: &TextDocumentItem,
) -> Option<Diagnostic> {
    let is_grouped = select_query.soulution_modifier()?.group_clause().is_some();
    let select_clause = select_query.select_clause()?;
    let is_star_selection = select_clause.is_star_selection();
    if !is_grouped || !is_star_selection {
        return None;
    }
    let star_token = select_clause.syntax().last_token()?;
    Some(Diagnostic {
        range: Range::from_byte_offset_range(star_token.text_range(), &document.text).expect("The star text range should be in text"),
        severity: DiagnosticSeverity::Error,
        code: Some((*CODE).clone()),
                source: Some("qlue-ls".to_string()),
        message: "Selecting * is not allowed when using GROUP BY!\nSelect grouped variables or aggregates.".to_string(),
        data: None
    }
    )
}
