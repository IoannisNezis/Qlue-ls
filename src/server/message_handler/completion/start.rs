use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::CompletionContext;

pub(super) async fn completions(_context: CompletionContext) -> Vec<CompletionItem> {
    vec![
        CompletionItem::new(
            "SELECT",
            "Select query",
            "SELECT ${1:*} WHERE {\n  $0\n}",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ),
        CompletionItem::new(
            "PREFIX",
            "Declare a namespace",
            "PREFIX ${1:namespace}: <${0:iri}>",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ),
        CompletionItem::new(
            "BASE",
            "Set the Base URI",
            "BASE <${0}>",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        ),
    ]
}
