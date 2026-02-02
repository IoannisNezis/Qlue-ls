mod core;
mod utils;
use crate::server::{
    Server,
    configuration::{FormatSettings, Settings},
    lsp::{
        FormattingOptions, FormattingRequest, FormattingResponse,
        errors::LSPError,
        textdocument::{Range, TextDocumentItem, TextEdit},
    },
    message_handler::{
        code_action::same_subject::contract_all_triple_groups,
    },
};
use core::format_document as format_document_core;
use futures::lock::Mutex;
use ll_sparql_parser::{SyntaxNode, parse};
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

/// Main entry point for formatting. Handles contract_triples as a second pass.
fn format_document(
    document: &TextDocumentItem,
    root: SyntaxNode,
    options: &FormattingOptions,
    settings: &FormatSettings,
) -> Result<Vec<TextEdit>, LSPError> {
    if settings.contract_triples {
        format_with_contraction(document, root, options, settings)
    } else {
        format_document_core(document, root, options, settings)
    }
}

/// Format with triple contraction enabled.
///
/// This function:
/// 1. Formats the document WITHOUT contract_triples first (to normalize whitespace)
/// 2. Applies formatting edits to get formatted text
/// 3. Re-parses the formatted document
/// 4. Groups all triples by subject and generates contracted output
/// 5. Returns a single edit replacing the entire document
fn format_with_contraction(
    document: &TextDocumentItem,
    root: SyntaxNode,
    options: &FormattingOptions,
    settings: &FormatSettings,
) -> Result<Vec<TextEdit>, LSPError> {
    // 1. Format WITHOUT contract_triples first
    let mut temp_settings = settings.clone();
    temp_settings.contract_triples = false;
    let format_edits = format_document_core(document, root, options, &temp_settings)?;

    // 2. Apply formatting edits to get formatted text
    let mut formatted_doc = document.clone();
    formatted_doc.apply_text_edits(format_edits);

    // 3. Re-parse the formatted document
    let (formatted_root, _) = parse(&formatted_doc.text);

    // 4. Apply contractions
    let contraction_edits = contract_all_triple_groups(&formatted_doc, formatted_root, settings)?
        .and_then(|action| action.edit.changes)
        .and_then(|mut changes| changes.remove(&document.uri))
        .unwrap_or_default();
    formatted_doc.apply_text_edits(contraction_edits);

    // 5. Return single edit replacing entire document
    Ok(vec![TextEdit::new(
        Range::new(0, 0, u32::MAX, u32::MAX),
        &formatted_doc.text,
    )])
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
