//! Add aggregate result code action

use std::collections::HashSet;

use ll_sparql_parser::{
    ast::{AstNode, SelectQuery, Var},
    syntax_kind::SyntaxKind,
};

use crate::server::lsp::{
    textdocument::{Position, Range, TextDocumentItem, TextEdit},
    CodeAction,
};

pub(super) fn code_actions(var: &Var, document: &TextDocumentItem) -> Option<Vec<CodeAction>> {
    let select_query = match var
        .syntax()
        .ancestors()
        .nth(2)
        .map(|grand_parent| grand_parent.kind())?
    {
        SyntaxKind::SubSelect => var.syntax().ancestors().skip(3).find_map(SelectQuery::cast),
        _ => var.syntax().ancestors().find_map(SelectQuery::cast),
    }?;
    let group_vars: Option<HashSet<String>> = select_query
        .soulution_modifier()
        .and_then(|solution_modifier| solution_modifier.group_clause())
        .map(|group_clause| {
            HashSet::from_iter(
                group_clause
                    .visible_variables()
                    .iter()
                    .map(|var| var.text()),
            )
        });
    if group_vars.is_some_and(|vars| !vars.contains(&var.var_name())) {
        let end = Position::from_byte_index(
            select_query
                .select_clause()?
                .syntax()
                .text_range()
                .end()
                .into(),
            &document.text,
        )?;
        let last_child = select_query
            .select_clause()?
            .syntax()
            .last_child_or_token()?;
        let aggregates = [
            (
                "Aggregate Count",
                &format!("(COUNT({}) AS ?count_{})", var.text(), var.var_name()),
            ),
            (
                "Aggregate Sum",
                &format!("(SUM({}) AS ?sum_{})", var.text(), var.var_name()),
            ),
            (
                "Aggregate Min",
                &format!("(MIN({}) AS ?min_{})", var.text(), var.var_name()),
            ),
            (
                "Aggregate Max",
                &format!("(MAX({}) AS ?max_{})", var.text(), var.var_name()),
            ),
            (
                "Aggregate Avg",
                &format!("(AVG({}) AS ?avg_{})", var.text(), var.var_name()),
            ),
            (
                "Aggregate Sample",
                &format!("(SAMPLE({}) AS ?sample_{})", var.text(), var.var_name()),
            ),
            (
                "Aggregate Group Concat",
                &format!(
                    "(GROUP_CONCAT(DISTINCT {}; SEPARATOR = \", \") as ?group_concat_{})",
                    var.text(),
                    var.var_name()
                ),
            ),
        ];
        return Some(
            aggregates
                .into_iter()
                .map(|(name, insert)| {
                    let mut ca = CodeAction::new(name, None);

                    if last_child.kind() == SyntaxKind::Star {
                        ca.add_edit(
                            &document.uri,
                            TextEdit::new(
                                Range::new(end.line, end.character - 1, end.line, end.character),
                                &insert,
                            ),
                        );
                    } else {
                        ca.add_edit(
                            &document.uri,
                            TextEdit::new(Range { start: end, end }, &format!(" {}", insert)),
                        );
                    }
                    ca
                })
                .collect(),
        );
    }
    None
}
