pub mod invalid_projection_variable;
pub mod uncompacted_uri;
pub mod undeclared_prefix;
pub mod ungrouped_select_variable;
pub mod unused_prefix;

use crate::server::{
    lsp::{errors::LSPError, DiagnosticRequest, DiagnosticResponse},
    Server,
};
use futures::lock::Mutex;
use ll_sparql_parser::{
    ast::{AstNode, QueryUnit},
    parse,
};
use std::rc::Rc;

pub(super) async fn handle_diagnostic_request(
    server_rc: Rc<Mutex<Server>>,
    request: DiagnosticRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let document = server
        .state
        .get_document(&request.params.text_document.uri)?;
    let ast = QueryUnit::cast(parse(&document.text)).ok_or(LSPError::new(
        crate::server::lsp::errors::ErrorCode::InternalError,
        "diagnostics are currently only supported for query operations",
    ))?;
    let mut diagnostic_accu = Vec::new();
    macro_rules! add {
        ($diagnostic_provider:path) => {
            if let Some(diagnostics) = $diagnostic_provider(document, &ast, &server) {
                diagnostic_accu.extend(diagnostics);
            }
        };
    }
    add!(unused_prefix::diagnostics);
    add!(undeclared_prefix::diagnostics);
    add!(uncompacted_uri::diagnostics);
    add!(uncompacted_uri::diagnostics);
    add!(ungrouped_select_variable::diagnostics);
    add!(invalid_projection_variable::diagnostics);

    server.send_message(DiagnosticResponse::new(request.get_id(), diagnostic_accu))
}
