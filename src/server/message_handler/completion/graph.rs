use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::{context::CompletionContext, error::CompletionError};

pub(super) fn completions(
    _context: CompletionContext,
) -> Result<Vec<CompletionItem>, CompletionError> {
    Ok(vec![CompletionItem::new(
        "<graph>",
        Some("hier könnte ihr Graph stehen".to_string()),
        None,
        "<graph>",
        CompletionItemKind::Value,
        InsertTextFormat::PlainText,
        None,
    )])
}
