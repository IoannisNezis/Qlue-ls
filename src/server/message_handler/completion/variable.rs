use super::{error::CompletionError, CompletionContext};
use crate::server::lsp::{
    CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
};
use ll_sparql_parser::{
    ast::{AstNode, PrefixedName, QueryUnit},
    syntax_kind::SyntaxKind,
};

pub(super) fn completions(context: CompletionContext) -> Result<CompletionList, CompletionError> {
    let query_unit = QueryUnit::cast(context.tree).ok_or(CompletionError::ResolveError(
        "Could not cast to QueryUnit".to_string(),
    ))?;
    let select_query = query_unit
        .select_query()
        .ok_or(CompletionError::ResolveError(
            "Could not find SelectQuery".to_string(),
        ))?;

    let mut suggestions: Vec<CompletionItem> = select_query
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
        .collect();
    if let Some(prefixed_name) = match context.location {
        super::CompletionLocation::Object(_) => context.anchor_token.and_then(|token| {
            (token.kind() == SyntaxKind::QuestionMark)
                .then_some(token.prev_token().and_then(|token| {
                    token
                        .prev_token()
                        .and_then(|token| token.parent().and_then(|node| PrefixedName::cast(node)))
                }))
                .flatten()
        }),
        _ => None,
    } {
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
                Some("1".to_string()),
                &format!("{} ", object_name),
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
