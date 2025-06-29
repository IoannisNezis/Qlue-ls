use ll_sparql_parser::syntax_kind::SyntaxKind;

use crate::server::{
    lsp::{
        Command, CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
    },
    message_handler::completion::{CompletionEnvironment, CompletionError},
};

pub(super) fn completions(
    context: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    Ok(
        if context
            .anchor_token
            .is_some_and(|anchor| anchor.kind() == SyntaxKind::BY)
        {
            CompletionList {
                is_incomplete: false,
                item_defaults: Some(ItemDefaults {
                    insert_text_format: Some(InsertTextFormat::Snippet),
                    data: None,
                    commit_characters: None,
                    edit_range: None,
                }),
                items: vec![
                    CompletionItem {
                        label: "DESC".to_string(),
                        label_details: None,
                        kind: CompletionItemKind::Snippet,
                        detail: Some("Order descending".to_string()),
                        sort_text: Some("00000".to_string()),
                        filter_text: None,
                        insert_text: Some("DESC($0)".to_string()),
                        text_edit: None,
                        insert_text_format: None,
                        additional_text_edits: None,
                        command: Some(Command {
                            title: "triggerNewCompletion".to_string(),
                            command: "triggerNewCompletion".to_string(),
                            arguments: None,
                        }),
                    },
                    CompletionItem {
                        label: "ASC".to_string(),
                        label_details: None,
                        kind: CompletionItemKind::Snippet,
                        detail: Some("Order Ascending".to_string()),
                        sort_text: Some("00001".to_string()),
                        filter_text: None,
                        insert_text: Some("ASC($0)".to_string()),
                        text_edit: None,
                        insert_text_format: None,
                        additional_text_edits: None,
                        command: Some(Command {
                            title: "triggerNewCompletion".to_string(),
                            command: "triggerNewCompletion".to_string(),
                            arguments: None,
                        }),
                    },
                ],
            }
        } else {
            CompletionList {
                is_incomplete: false,
                item_defaults: None,
                items: Vec::new(),
            }
        },
    )
}
