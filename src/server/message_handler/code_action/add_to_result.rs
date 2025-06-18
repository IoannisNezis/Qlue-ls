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
    if !result_vars.contains(&token.to_string()) {
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
