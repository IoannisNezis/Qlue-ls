use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::{
    ast::{AstNode, Prologue},
    parse,
};

use crate::server::{
    Server,
    lsp::{
        FoldingRange, FoldingRangeKind, FoldingRangeRequest, FoldingRangeResponse,
        errors::LSPError, textdocument::Range,
    },
};

pub(super) async fn handle_folding_range_request(
    server_rc: Rc<Mutex<Server>>,
    request: FoldingRangeRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let mut result = vec![];
    let document = server.state.get_document(request.get_document_uri())?;
    let tree = parse(&document.text);
    if let Some(prologue) = tree
        .first_child()
        .and_then(|child| child.first_child().and_then(Prologue::cast))
    {
        let range =
            Range::from_byte_offset_range(prologue.syntax().text_range(), &document.text).unwrap();
        result.push(FoldingRange {
            start_line: range.start.line,
            end_line: range.end.line,
            start_character: None,
            end_character: None,
            kind: Some(FoldingRangeKind::Imports),
            collapsed_text: Some(prologue.text()),
        });
    }
    let mut response = FoldingRangeResponse::new(request.get_id());
    response.set_result(result);
    server.send_message(response)
}
