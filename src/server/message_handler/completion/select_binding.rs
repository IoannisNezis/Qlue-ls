use super::{error::CompletionError, CompletionContext, CompletionLocation};
use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};
use ll_sparql_parser::{ast::AstNode, syntax_kind::SyntaxKind};
use std::collections::HashSet;

pub(super) fn completions(
    context: CompletionContext,
) -> Result<Vec<CompletionItem>, CompletionError> {
    if let CompletionLocation::SelectBinding(select_clause) = &context.location {
        let mut res = Vec::new();
        if context.continuations.contains(&SyntaxKind::DISTINCT) {
            res.append(&mut vec![
                CompletionItem::new(
                    "DISTINCT",
                    Some("Ensure unique results".to_string()),
                    None,
                    "DISTINCT ",
                    CompletionItemKind::Keyword,
                    InsertTextFormat::PlainText,
                    None,
                ),
                CompletionItem::new(
                    "REDUCED",
                    Some("Permit elimination of some non-distinct solutions".to_string()),
                    None,
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
                Some("variable".to_string()),
                None,
                &format!("{} ", var),
                CompletionItemKind::Variable,
                InsertTextFormat::PlainText,
                None,
            )
        }));
        Ok(res)
    } else {
        Err(CompletionError::ResolveError(format!(
            "select binding completions was called with location: {:?}",
            context.location
        )))
    }
}
