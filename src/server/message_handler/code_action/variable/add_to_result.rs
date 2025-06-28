//! Add to result code action
//! Add variable to `SelectClause`

use std::collections::HashSet;

use ll_sparql_parser::{
    ast::{AstNode, SelectQuery, Var},
    syntax_kind::SyntaxKind,
};

use crate::server::lsp::{
    textdocument::{Position, Range, TextDocumentItem, TextEdit},
    CodeAction,
};

pub(super) fn code_action(var: &Var, document: &TextDocumentItem) -> Option<CodeAction> {
    let select_query = match var
        .syntax()
        .ancestors()
        .nth(2)
        .map(|grand_parent| grand_parent.kind())?
    {
        SyntaxKind::SubSelect => var.syntax().ancestors().skip(3).find_map(SelectQuery::cast),
        _ => var.syntax().ancestors().find_map(SelectQuery::cast),
    }?;

    let result_vars: HashSet<String> = HashSet::from_iter(
        select_query
            .select_clause()?
            .variables()
            .iter()
            .map(|var| var.text()),
    );
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
    if !result_vars.contains(&var.text())
        && group_vars.map_or(true, |vars| vars.contains(&var.text()))
    {
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
        let mut ca = CodeAction::new("Add to result", None);
        if last_child.kind() == SyntaxKind::Star {
            ca.add_edit(
                &document.uri,
                TextEdit::new(
                    Range::new(end.line, end.character - 1, end.line, end.character),
                    &var.text(),
                ),
            );
        } else {
            ca.add_edit(
                &document.uri,
                TextEdit::new(Range { start: end, end }, &format!(" {}", var.text())),
            );
        }
        return Some(ca);
    }
    None
}
