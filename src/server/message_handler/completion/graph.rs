use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::context::CompletionContext;

pub(super) fn completions(_context: CompletionContext) -> Vec<CompletionItem> {
    vec![CompletionItem::new(
        "<graph>",
        Some("hier könnte ihr Graph stehen".to_string()),
        None,
        "<graph>",
        CompletionItemKind::Value,
        InsertTextFormat::PlainText,
        None,
    )]
}
