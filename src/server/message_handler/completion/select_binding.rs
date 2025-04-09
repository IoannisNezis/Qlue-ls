use super::{error::CompletionError, CompletionContext, CompletionLocation};
use crate::server::lsp::{
    CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
};
use ll_sparql_parser::{ast::AstNode, syntax_kind::SyntaxKind};
use std::collections::HashSet;

pub(super) fn completions(context: CompletionContext) -> Result<CompletionList, CompletionError> {
    if let CompletionLocation::SelectBinding(select_clause) = &context.location {
        let mut items = Vec::new();
        if context.continuations.contains(&SyntaxKind::DISTINCT) {
            items.append(&mut vec![
                CompletionItem::new(
                    "DISTINCT",
                    Some("Ensure unique results".to_string()),
                    None,
                    "DISTINCT ",
                    CompletionItemKind::Keyword,
                    None,
                ),
                CompletionItem::new(
                    "REDUCED",
                    Some("Permit elimination of some non-distinct solutions".to_string()),
                    None,
                    "REDUCED ",
                    CompletionItemKind::Keyword,
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
        items.extend(vars.into_iter().map(|var| {
            CompletionItem::new(
                &var,
                Some("variable".to_string()),
                None,
                &format!("{} ", var),
                CompletionItemKind::Variable,
                None,
            )
        }));
        Ok(CompletionList {
            is_incomplete: false,
            item_defaults: Some(ItemDefaults {
                edit_range: None,
                commit_characters: None,
                data: None,
                insert_text_format: Some(InsertTextFormat::PlainText),
            }),
            items,
        })
    } else {
        Err(CompletionError::ResolveError(format!(
            "select binding completions was called with location: {:?}",
            context.location
        )))
    }
}
