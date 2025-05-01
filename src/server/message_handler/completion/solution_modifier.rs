use super::{error::CompletionError, CompletionContext};
use crate::server::lsp::{
    Command, CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
};
use ll_sparql_parser::syntax_kind::SyntaxKind::*;

pub(super) fn completions(context: CompletionContext) -> Result<CompletionList, CompletionError> {
    let mut items = Vec::new();
    if context.continuations.contains(&SolutionModifier) {
        items.push(CompletionItem {
            label: "GROUP BY".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Group the results".to_string()),
            sort_text: None,
            insert_text: Some("GROUP BY $0".to_string()),
            text_edit: None,
            insert_text_format: None,
            additional_text_edits: None,
            command: Some(Command {
                title: "triggerNewCompletion".to_string(),
                command: "triggerNewCompletion".to_string(),
                arguments: None,
            }),
        });
    }
    if context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&HavingClause)
    {
        items.push(CompletionItem {
            command: None,
            label: "HAVING".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Filter Groups".to_string()),
            sort_text: None,
            insert_text: Some("HAVING $0".to_string()),
            text_edit: None,
            insert_text_format: None,
            additional_text_edits: None,
        });
    }
    if context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&OrderClause)
    {
        items.push(CompletionItem {
            label: "ORDER BY".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Sort the results".to_string()),
            sort_text: None,
            insert_text: Some("ORDER BY ${1|ASC,DESC|} ($0)".to_string()),
            text_edit: None,
            insert_text_format: None,
            additional_text_edits: None,

            command: Some(Command {
                title: "triggerNewCompletion".to_string(),
                command: "triggerNewCompletion".to_string(),
                arguments: None,
            }),
        });
    }
    if context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&LimitClause)
        || context.continuations.contains(&LimitOffsetClauses)
    {
        items.push(CompletionItem {
            label: "LIMIT".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Limit the results".to_string()),
            sort_text: None,
            insert_text: Some("LIMIT $0".to_string()),
            text_edit: None,
            insert_text_format: None,
            additional_text_edits: None,
            command: None,
        });
    }
    if context.continuations.contains(&SolutionModifier)
        || context.continuations.contains(&OffsetClause)
        || context.continuations.contains(&LimitOffsetClauses)
    {
        items.push(CompletionItem {
            label: "OFFSET".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("OFFSET the results".to_string()),
            sort_text: None,
            insert_text: Some("OFFSET $0".to_string()),
            text_edit: None,
            insert_text_format: None,
            additional_text_edits: None,
            command: None,
        });
    }
    return Ok(CompletionList {
        is_incomplete: false,
        item_defaults: Some(ItemDefaults {
            insert_text_format: Some(InsertTextFormat::Snippet),
            data: None,
            commit_characters: None,
            edit_range: None,
        }),
        items,
    });
}
