use std::rc::Rc;

use crate::{
    Server,
    server::{
        lsp::{
            DocumentHighlight, DocumentHighlightKind, DocumentHighlightRequest,
            DocumentHighlightResponse,
            errors::{ErrorCode, LSPError},
            textdocument::Range,
        },
        message_handler::common::find_variable_occurrences,
    },
};
use futures::lock::Mutex;
use ll_sparql_parser::ast::{AstNode, Var};

pub(super) async fn handle_document_highlight_request(
    server_rc: Rc<Mutex<Server>>,
    request: DocumentHighlightRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let document = server.state.get_document(request.get_document_uri())?;
    let root = server
        .state
        .get_cached_parse_tree(request.get_document_uri())?
        .tree;
    let offset = request
        .get_position()
        .byte_index(&document.text)
        .ok_or_else(|| {
            LSPError::new(
                ErrorCode::InvalidParams,
                "The hover position is not inside the text document",
            )
        })?;

    let seleced_token = root.token_at_offset(offset).left_biased().ok_or_else(|| {
        LSPError::new(
            ErrorCode::InvalidParams,
            "The hover position is not inside the text document",
        )
    })?;

    let mut response = DocumentHighlightResponse::new(request.get_id());
    if let Some(var) = seleced_token.parent().and_then(Var::cast) {
        let vars = find_variable_occurrences(&var);
        let ranges: Vec<_> = vars
            .into_iter()
            .map(|var| DocumentHighlight {
                range: Range::from_byte_offset_range(var.syntax().text_range(), &document.text)
                    .unwrap(),
                kind: Some(DocumentHighlightKind::Text),
            })
            .collect();
        response.set_highlights(ranges);
    }

    server.send_message(response)
}
