mod documentation;
mod iri;

use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::{parse_query, syntax_kind::SyntaxKind, TokenAtOffset};

use crate::server::{
    lsp::{
        errors::{ErrorCode, LSPError},
        HoverRequest, HoverResponse,
    },
    Server,
};

pub(super) async fn handle_hover_request(
    server_rc: Rc<Mutex<Server>>,
    request: HoverRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let mut hover_response = HoverResponse::new(request.get_id());
    let document = server.state.get_document(request.get_document_uri())?;
    let root = parse_query(&document.text);
    let offset = request
        .get_position()
        .to_byte_index(&document.text)
        .ok_or_else(|| {
            LSPError::new(
                ErrorCode::InvalidParams,
                "The hover position is not inside the text document",
            )
        })? as u32;
    if let TokenAtOffset::Single(token) = root.token_at_offset(offset.into()) {
        if let Some(content) = match token.kind() {
            SyntaxKind::PNAME_LN | SyntaxKind::PNAME_NS | SyntaxKind::IRIREF => {
                iri::hover(&*server, token).await?
            }
            other => documentation::get_docstring_for_kind(other),
        } {
            hover_response.set_markdown_content(content.to_string());
        }
    }
    server.send_message(hover_response)
}
