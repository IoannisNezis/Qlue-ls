mod core;
mod utils;
use crate::server::{
    Server,
    configuration::Settings,
    lsp::{
        FormattingOptions, FormattingRequest, FormattingResponse, errors::LSPError,
        textdocument::TextDocumentItem,
    },
};
use core::*;
use futures::lock::Mutex;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;

pub(super) async fn handle_format_request(
    server_rc: Rc<Mutex<Server>>,
    request: FormattingRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let document = server.state.get_document(request.get_document_uri())?;
    let edits = format_document(document, request.get_options(), &server.settings.format)?;
    server.send_message(FormattingResponse::new(request.get_id(), edits))
}

#[wasm_bindgen]
pub fn format_raw(text: String) -> Result<String, String> {
    let settings = Settings::new();
    let mut document = TextDocumentItem::new("tmp", &text);
    let edits = format_document(
        &document,
        // &tree,
        &FormattingOptions {
            tab_size: 2,
            insert_spaces: true,
        },
        &settings.format,
    )
    .map_err(|err| err.message)?;
    document.apply_text_edits(edits);
    Ok(document.text)
}

#[cfg(test)]
mod tests;
