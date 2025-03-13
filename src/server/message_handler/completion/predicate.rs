use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::CompletionContext;

pub(super) fn completions(_context: CompletionContext) -> Vec<CompletionItem> {
    vec![CompletionItem::new(
        "predicate filler",
        "Hier könnte ihre predicate completion stehen",
        "<predicate> ",
        CompletionItemKind::Value,
        InsertTextFormat::PlainText,
    )]
}
