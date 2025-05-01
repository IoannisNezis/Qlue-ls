use std::rc::Rc;

use futures::lock::Mutex;

use super::error::CompletionError;
use crate::server::{
    lsp::{CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat},
    Server,
};

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
) -> Result<CompletionList, CompletionError> {
    let server = server_rc.lock().await;
    let default_backend = server.state.get_default_backend();
    Ok(CompletionList {
        is_incomplete: false,
        item_defaults: None,
        items: server
            .state
            .get_all_backends()
            .into_iter()
            .filter(|backend| {
                default_backend
                    .map(|default| backend.name != default.name)
                    .unwrap_or(true)
            })
            .map(|backend| CompletionItem {command: None,
                label: backend.name.clone(),
                label_details: None,
                kind: CompletionItemKind::Value,
                detail: Some(backend.url.clone()),
                sort_text: None,
                insert_text: Some(format!("<{}>", backend.url)),
                text_edit: None,
                insert_text_format: Some(InsertTextFormat::PlainText),
                additional_text_edits: None,
            })
            .collect(),
    })
}
