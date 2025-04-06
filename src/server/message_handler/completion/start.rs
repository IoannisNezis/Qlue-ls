use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::{error::CompletionError, CompletionContext};

pub(super) async fn completions(
    _context: CompletionContext,
) -> Result<Vec<CompletionItem>, CompletionError> {
    Ok(vec![
        CompletionItem::new(
            "SELECT",
            Some("Select query".to_string()),
            None,
            "SELECT ${1:*} WHERE {\n  $0\n}",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ),
        CompletionItem::new(
            "PREFIX",
            Some("Declare a namespace".to_string()),
            None,
            "PREFIX ${1:namespace}: <${0:iri}>",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ),
        CompletionItem::new(
            "BASE",
            Some("Set the Base URI".to_string()),
            None,
            "BASE <${0}>",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ),
    ])
}
