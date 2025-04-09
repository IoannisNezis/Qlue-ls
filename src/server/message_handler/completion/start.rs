use crate::server::lsp::{
    CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
};

use super::{error::CompletionError, CompletionContext};

pub(super) async fn completions(
    _context: CompletionContext,
) -> Result<CompletionList, CompletionError> {
    Ok(CompletionList {
        is_incomplete: false,
        item_defaults: Some(ItemDefaults {
            edit_range: None,
            commit_characters: None,
            data: None,
            insert_text_format: Some(InsertTextFormat::Snippet),
        }),
        items: vec![
            CompletionItem::new(
                "SELECT",
                Some("Select query".to_string()),
                None,
                "SELECT ${1:*} WHERE {\n  $0\n}",
                CompletionItemKind::Snippet,
                None,
            ),
            CompletionItem::new(
                "PREFIX",
                Some("Declare a namespace".to_string()),
                None,
                "PREFIX ${1:namespace}: <${0:iri}>",
                CompletionItemKind::Snippet,
                None,
            ),
            CompletionItem::new(
                "BASE",
                Some("Set the Base URI".to_string()),
                None,
                "BASE <${0}>",
                CompletionItemKind::Snippet,
                None,
            ),
        ],
    })
}
