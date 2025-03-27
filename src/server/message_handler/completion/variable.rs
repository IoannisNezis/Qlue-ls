use super::CompletionContext;
use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};
use ll_sparql_parser::ast::{AstNode, QueryUnit};

pub(super) fn completions(context: CompletionContext) -> Vec<CompletionItem> {
    let query_unit = QueryUnit::cast(context.tree).unwrap();
    if let Some(select_query) = query_unit.select_query() {
        select_query
            .variables()
            .iter()
            .map(|var| {
                let var_text = var.var_name();
                CompletionItem::new(
                    &var_text,
                    Some("Variable".to_string()),
                    None,
                    &format!("{} ", var_text),
                    CompletionItemKind::Variable,
                    InsertTextFormat::PlainText,
                    None,
                )
            })
            .collect()
    } else {
        vec![]
    }
}
