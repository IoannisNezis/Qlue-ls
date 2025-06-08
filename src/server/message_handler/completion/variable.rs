use std::collections::HashSet;

use super::{error::CompletionError, CompletionEnvironment, CompletionLocation};
use crate::server::lsp::{
    Command, CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
};
use ll_sparql_parser::ast::{AstNode, PrefixedName, Var};

pub(super) fn completions(
    context: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let suffix = match context.location {
        CompletionLocation::Object(_)
        | CompletionLocation::Subject
        | CompletionLocation::Predicate(_)
        | CompletionLocation::BlankNodeProperty(_)
        | CompletionLocation::BlankNodeObject(_) => " ",
        _ => "",
    };
    let mut suggestions: Vec<CompletionItem> = HashSet::<String>::from_iter(
        context
            .tree
            .descendants()
            .filter_map(Var::cast)
            .map(|var| var.var_name()),
    )
    .into_iter()
    .map(|var| CompletionItem {
        insert_text: Some(format!("{}{}", var, suffix)),
        label: var,
        label_details: None,
        detail: Some("Variable".to_string()),
        kind: CompletionItemKind::Variable,
        sort_text: None,
        filter_text: None,
        text_edit: None,
        insert_text_format: Some(InsertTextFormat::PlainText),
        additional_text_edits: None,
        command: match context.location {
            CompletionLocation::Subject
            | CompletionLocation::Predicate(_)
            | CompletionLocation::BlankNodeProperty(_) => Some(Command {
                title: "triggerNewCompletion".to_string(),
                command: "triggerNewCompletion".to_string(),
                arguments: None,
            }),
            _ => None,
        },
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
    environment: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let mut variable_completions = completions(environment)?;
    for item in variable_completions.items.iter_mut() {
        item.insert_text = item.insert_text.as_ref().map(|text| format!("?{}", text));
        item.label = format!("?{}", item.label);
        if item.sort_text.is_none() {
            item.sort_text = Some(format!("{:0>5}", 1));
        }
    }
    Ok(variable_completions)
}
