use super::{error::CompletionError, CompletionContext};
use crate::server::lsp::{
    CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
};
use ll_sparql_parser::ast::{AstNode, QueryUnit};

pub(super) fn completions(context: CompletionContext) -> Result<CompletionList, CompletionError> {
    let query_unit = QueryUnit::cast(context.tree).ok_or(CompletionError::ResolveError(
        "Could not cast to QueryUnit".to_string(),
    ))?;
    let select_query = query_unit
        .select_query()
        .ok_or(CompletionError::ResolveError(
            "Could not find SelectQuery".to_string(),
        ))?;
    Ok(CompletionList {
        is_incomplete: false,
        item_defaults: Some(ItemDefaults {
            edit_range: None,
            commit_characters: None,
            data: None,
            insert_text_format: Some(InsertTextFormat::PlainText),
        }),
        items: select_query
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
                    None,
                )
            })
            .collect(),
    })
}
