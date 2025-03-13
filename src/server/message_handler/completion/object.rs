use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::CompletionContext;

pub(super) fn completions(_context: CompletionContext) -> Vec<CompletionItem> {
    vec![CompletionItem::new(
        "object filler",
        "Hier könnte ihre object completion stehen",
        "<object> ",
        CompletionItemKind::Value,
        InsertTextFormat::PlainText,
    )]
}
