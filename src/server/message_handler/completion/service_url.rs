use super::error::CompletionError;
use crate::server::{
    lsp::{CompletionItem, CompletionItemKind, InsertTextFormat},
    Server,
};

pub(super) fn completions(server: &Server) -> Result<Vec<CompletionItem>, CompletionError> {
    let default_backend = server.state.get_default_backend();
    Ok(server
        .state
        .get_all_backends()
        .into_iter()
        .filter(|backend| {
            default_backend
                .map(|default| backend.name != default.name)
                .unwrap_or(true)
        })
        .map(|backend| {
            CompletionItem::new(
                &backend.name,
                Some(backend.url.clone()),
                None,
                &format!("<{}>", backend.url),
                CompletionItemKind::Value,
                InsertTextFormat::PlainText,
                None,
            )
        })
        .collect())
}
