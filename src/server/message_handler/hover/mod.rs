mod documentation;
mod iri;

use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::{TokenAtOffset, parse_query, syntax_kind::SyntaxKind};

use crate::server::{
    Server,
    lsp::{
        HoverRequest, HoverResponse,
        errors::{ErrorCode, LSPError},
    },
};

pub(super) async fn handle_hover_request(
    server_rc: Rc<Mutex<Server>>,
    request: HoverRequest,
) -> Result<(), LSPError> {
    let document_text = {
        let server = server_rc.lock().await;
        let document = server.state.get_document(request.get_document_uri())?;
        document.text.clone()
    };
    let mut hover_response = HoverResponse::new(request.get_id());
    let root = parse_query(&document_text);
    let offset = request
        .get_position()
        .byte_index(&document_text)
        .ok_or_else(|| {
            LSPError::new(
                ErrorCode::InvalidParams,
                "The hover position is not inside the text document",
            )
        })?;
    if let TokenAtOffset::Single(token) = root.token_at_offset(offset) {
        if let Some(content) = match token.kind() {
            SyntaxKind::PNAME_LN | SyntaxKind::PNAME_NS | SyntaxKind::IRIREF => {
                iri::hover(server_rc.clone(), root, token).await?
            }
            other => documentation::get_docstring_for_kind(other),
        } {
            hover_response.set_markdown_content(content.to_string());
        }
    }
    server_rc.lock().await.send_message(hover_response)
}
