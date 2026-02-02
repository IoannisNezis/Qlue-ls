mod core;
mod utils;
use crate::server::{
    Server,
    configuration::{FormatSettings, Settings},
    lsp::{
        FormattingOptions, FormattingRequest, FormattingResponse, errors::LSPError,
        textdocument::TextDocumentItem,
    },
};
use core::*;
use futures::lock::Mutex;
use ll_sparql_parser::parse;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;

pub(super) async fn handle_format_request(
    server_rc: Rc<Mutex<Server>>,
    request: FormattingRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let document = server.state.get_document(request.get_document_uri())?;
    let root = server
        .state
        .get_cached_parse_tree(request.get_document_uri())?;
    let edits = format_document(
        document,
        root,
        request.get_options(),
        &server.settings.format,
    )?;
    server.send_message(FormattingResponse::new(request.get_id(), edits))
}

#[wasm_bindgen]
pub fn format_raw(text: String) -> Result<String, String> {
    let settings = Settings::new();
    let mut document = TextDocumentItem::new("tmp", &text);
    let (root, _) = parse(&text);
    let edits = format_document(
        &document,
        root,
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

/// Format SPARQL text with custom settings.
///
/// This function is intended for testing and allows specifying custom format settings.
/// Also checks for edit collisions (overlapping edits) which would indicate a formatter bug.
pub fn format_with_settings(
    text: String,
    format_settings: FormatSettings,
) -> Result<String, String> {
    let mut document = TextDocumentItem::new("tmp", &text);
    let (root, _) = parse(&text);
    let edits = format_document(
        &document,
        root,
        &FormattingOptions {
            tab_size: format_settings.tab_size.unwrap_or(2),
            insert_spaces: format_settings.insert_spaces.unwrap_or(true),
        },
        &format_settings,
    )
    .map_err(|err| err.message)?;

    // Check for overlapping edits (indicates a formatter bug)
    for idx1 in 0..edits.len() {
        for idx2 in idx1 + 1..edits.len() {
            let a = &edits[idx1];
            let b = &edits[idx2];
            if a.overlaps(b) {
                return Err(format!("Edits overlap: {} vs {}", a, b));
            }
        }
    }

    document.apply_text_edits(edits);
    Ok(document.text)
}
