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

pub fn code_action(token: SyntaxToken, document: &TextDocumentItem) -> Option<CodeAction> {
    if let Some(select_clause) = token
        .parent_ancestors()
        .find(|ancestor| {
            ancestor.kind() == SyntaxKind::SelectQuery || ancestor.kind() == SyntaxKind::SubSelect
        })
        .and_then(|node| SelectQuery::cast(node).and_then(|sq| sq.select_clause()))
    {
        let result_vars: HashSet<String> = HashSet::from_iter(
            select_clause
                .variables()
                .iter()
                .map(|var| var.syntax().text().to_string()),
        );
        if !result_vars.contains(&token.to_string()) {
            if let Some(end) = Position::from_byte_index(
                select_clause.syntax().text_range().end().into(),
                &document.text,
            ) {
                if let Some(last_child) = select_clause.syntax().last_child_or_token() {
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
            }
        }
    }
    None
}
