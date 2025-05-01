use crate::server::lsp::{CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat};

use super::{context::CompletionContext, error::CompletionError};

pub(super) fn completions(_context: CompletionContext) -> Result<CompletionList, CompletionError> {
    Ok(CompletionList {
        is_incomplete: false,
        item_defaults: None,
        items: vec![CompletionItem {command: None,
            label: "<graph>".to_string(),
            label_details: None,
            kind: CompletionItemKind::Value,
            detail: Some("hier könnte ihr Graph stehen".to_string()),
            sort_text: None,
            insert_text: Some("<graph>".to_string()),
            text_edit: None,
            insert_text_format: Some(InsertTextFormat::PlainText),
            additional_text_edits: None,
        }],
    })
}
