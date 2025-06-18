//! Add to result code action
//! Add variabel to `SelectClause`

use std::collections::HashSet;

use ll_sparql_parser::{
    ast::{AstNode, SelectQuery},
    syntax_kind::SyntaxKind,
    SyntaxToken,
};

use crate::server::lsp::{
    textdocument::{Position, Range, TextDocumentItem, TextEdit},
    CodeAction,
};

pub(super) fn code_action(token: &SyntaxToken, document: &TextDocumentItem) -> Option<CodeAction> {
    assert!(matches!(token.kind(), SyntaxKind::VAR1 | SyntaxKind::VAR2));
    let select_query = match token
        .parent_ancestors()
        .nth(2)
        .map(|grand_parent| grand_parent.kind())?
    {
        SyntaxKind::SubSelect => token.parent_ancestors().skip(3).find_map(SelectQuery::cast),
        _ => token.parent_ancestors().find_map(SelectQuery::cast),
    }?;
    let result_vars: HashSet<String> = HashSet::from_iter(
        select_query
            .select_clause()?
            .variables()
            .iter()
            .map(|var| var.syntax().text().to_string()),
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
    if !result_vars.contains(&token.to_string())
        && group_vars.map_or(true, |vars| vars.contains(&token.to_string()))
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
                    &token.to_string(),
                ),
            );
        } else {
            ca.add_edit(
                &document.uri,
                TextEdit::new(Range { start: end, end }, &format!(" {}", token)),
            );
        }
        return Some(ca);
    }
    None
}
