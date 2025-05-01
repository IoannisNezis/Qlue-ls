use std::collections::HashSet;

use super::{error::CompletionError, CompletionContext, CompletionLocation};
use crate::server::lsp::{
    CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
};
use ll_sparql_parser::ast::{AstNode, PrefixedName, QueryUnit};

pub(super) fn completions(context: CompletionContext) -> Result<CompletionList, CompletionError> {
    let query_unit = QueryUnit::cast(context.tree).ok_or(CompletionError::ResolveError(
        "Could not cast to QueryUnit".to_string(),
    ))?;
    let select_query = query_unit
        .select_query()
        .ok_or(CompletionError::ResolveError(
            "Could not find SelectQuery".to_string(),
        ))?;
    let suffix = match context.location {
        CompletionLocation::Object(_)
        | CompletionLocation::Subject
        | CompletionLocation::Predicate(_)
        | CompletionLocation::BlankNodeProperty(_)
        | CompletionLocation::BlankNodeObject(_) => " ",
        _ => "",
    };
    let mut suggestions: Vec<CompletionItem> = HashSet::<String>::from_iter(
        select_query
            .variables()
            .into_iter()
            .map(|var| var.var_name()),
    )
    .iter()
    .map(|var| {
        CompletionItem::new(
            &var,
            Some("Variable".to_string()),
            None,
            &format!("{}{}", var, suffix),
            CompletionItemKind::Variable,
            None,
        )
    })
    .collect();
    if let Some(prefixed_name) = context
        .anchor_token
        .and_then(|token| token.parent())
        .and_then(PrefixedName::cast)
    {
        let object_name = prefixed_name
            .name()
            .strip_prefix("has")
            .map(|name| name.to_lowercase())
            .unwrap_or(prefixed_name.name());
        suggestions.insert(
            0,
            CompletionItem::new(
                &object_name,
                None,
                Some(format!("{:0>5}", 0)),
                &format!("{}{}", object_name, suffix),
                CompletionItemKind::Variable,
                None,
            ),
        );
    }
    Ok(CompletionList {
        is_incomplete: false,
        item_defaults: Some(ItemDefaults {
            edit_range: None,
            commit_characters: None,
            data: None,
            insert_text_format: Some(InsertTextFormat::PlainText),
        }),
        items: suggestions,
    })
}

pub(super) fn completions_transformed(
    context: CompletionContext,
) -> Result<CompletionList, CompletionError> {
    let mut variable_completions = completions(context)?;
    for item in variable_completions.items.iter_mut() {
        item.insert_text = item.insert_text.as_ref().map(|text| format!("?{}", text));
        item.label = format!("?{}", item.label);
        if item.sort_text.is_none() {
            item.sort_text = Some(format!("{:0>5}", 1));
        }
    }
    Ok(variable_completions)
}
