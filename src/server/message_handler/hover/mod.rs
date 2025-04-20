mod documentation;
mod iri;

use std::{cell::RefCell, rc::Rc};

use ll_sparql_parser::{parse_query, syntax_kind::SyntaxKind, TokenAtOffset};

use crate::server::{
    self,
    lsp::{
        errors::{ErrorCode, LSPError},
        HoverRequest, HoverResponse,
    },
    Server,
};

pub(super) async fn handle_hover_request(
    server_rc: Rc<RefCell<Server>>,
    request: HoverRequest,
) -> Result<(), LSPError> {
    let server = server_rc.borrow();
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
                iri::hover(server_rc.clone(), token).await?
            }
            other => documentation::get_docstring_for_kind(other),
        } {
            hover_response.set_markdown_content(content.to_string());
        }
    }
    server.send_message(hover_response)
}
