mod uncompacted_uri;
mod undeclared_prefix;
mod unused_prefix;
use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::parse_query;

use crate::server::{
    lsp::{errors::LSPError, DiagnosticRequest, DiagnosticResponse},
    Server,
};

pub(super) async fn handle_diagnostic_request(
    server_rc: Rc<Mutex<Server>>,
    request: DiagnosticRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let document = server
        .state
        .get_document(&request.params.text_document.uri)?;
    let parse_tree = parse_query(&document.text);

    let mut diagnostics = Vec::new();
    if let Some(unused_prefix_diagnostics) =
        unused_prefix::diagnostics(document, parse_tree.clone())
    {
        diagnostics.extend(unused_prefix_diagnostics);
    }

    if let Some(undeclared_prefix_diagnostics) =
        undeclared_prefix::diagnostics(document, parse_tree.clone())
    {
        diagnostics.extend(undeclared_prefix_diagnostics);
    }

    if let Some(uncompacted_uri_diagnostics) =
        uncompacted_uri::diagnostics(document, &*server, parse_tree)
    {
        diagnostics.extend(uncompacted_uri_diagnostics);
    }

    server.send_message(DiagnosticResponse::new(request.get_id(), diagnostics))
}

// fn undefined_select_binding(
//     server: &Server,
//     document: &TextDocumentItem,
// ) -> Result<impl Iterator<Item = Diagnostic>, LSPError> {
//     todo!()
// }
