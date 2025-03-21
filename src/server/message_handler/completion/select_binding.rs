use super::{CompletionContext, CompletionLocation};
use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};
use ll_sparql_parser::{ast::AstNode, syntax_kind::SyntaxKind};
use std::collections::HashSet;

pub(super) fn completions(context: CompletionContext) -> Vec<CompletionItem> {
    if let CompletionLocation::SelectBinding(select_clause) = &context.location {
        let mut res = Vec::new();
        if context.continuations.contains(&SyntaxKind::DISTINCT) {
            res.append(&mut vec![
                CompletionItem::new(
                    "DISTINCT",
                    "Ensure unique results",
                    "DISTINCT ",
                    CompletionItemKind::Keyword,
                    InsertTextFormat::PlainText,
                    None,
                ),
                CompletionItem::new(
                    "REDUCED",
                    "Permit elimination of some non-distinct solutions",
                    "REDUCED ",
                    CompletionItemKind::Keyword,
                    InsertTextFormat::PlainText,
                    None,
                ),
            ]);
        }
        let result_vars: HashSet<String> = HashSet::from_iter(
            select_clause
                .variables()
                .iter()
                .map(|var| var.syntax().text().to_string()),
        );
        let availible_vars: HashSet<String> =
            select_clause
                .select_query()
                .map_or(HashSet::new(), |select_query| {
                    HashSet::from_iter(
                        select_query
                            .variables()
                            .iter()
                            .map(|var| var.syntax().text().to_string()),
                    )
                });
        let vars = &availible_vars - &result_vars;
        res.extend(vars.into_iter().map(|var| {
            CompletionItem::new(
                &var,
                "variable",
                &format!("{} ", var),
                CompletionItemKind::Variable,
                InsertTextFormat::PlainText,
                None,
            )
        }));
        return res;
    } else {
        log::error!(
            "select binding completions was called with location: {:?}",
            context.location
        );
        return vec![];
    }
}
