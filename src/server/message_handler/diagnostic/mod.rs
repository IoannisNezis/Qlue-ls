mod auto_fix_prefixes;
pub mod duplicate_prefix_declaration;
pub mod groupby_star_selection;
pub mod invalid_projection_variable;
pub mod same_subject;
pub mod syntax_error;
pub mod uncompacted_uri;
pub mod undeclared_prefix;
pub mod ungrouped_select_variable;
pub mod unused_prefix_declaration;

use crate::server::{
    Server,
    lsp::{DiagnosticRequest, DiagnosticResponse, errors::LSPError},
};
use auto_fix_prefixes::{auto_fix_prefixes, client_support_workspace_edits};
use futures::lock::Mutex;
use ll_sparql_parser::ast::{AstNode, QueryUnit};
use std::rc::Rc;

#[tracing::instrument(skip_all, fields(id = %request.get_id(), uri = %request.params.text_document.uri))]
pub(super) async fn handle_diagnostic_request(
    server_rc: Rc<Mutex<Server>>,
    request: DiagnosticRequest,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
    let document = server
        .state
        .get_document(&request.params.text_document.uri)?;
    let tree = server
        .state
        .get_cached_parse_tree(&request.params.text_document.uri)?
        .tree;
    let ast = QueryUnit::cast(tree).ok_or(LSPError::new(
        crate::server::lsp::errors::ErrorCode::InternalError,
        "diagnostics are currently only supported for query operations",
    ))?;
    let mut diagnostic_accu = Vec::new();
    macro_rules! add_diagnostic {
        ($diagnostic_provider:path) => {
            if let Some(diagnostics) = $diagnostic_provider(document, &ast, &server) {
                diagnostic_accu.extend(diagnostics);
            }
        };
    }
    add_diagnostic!(unused_prefix_declaration::diagnostics);
    add_diagnostic!(undeclared_prefix::diagnostics);
    add_diagnostic!(uncompacted_uri::diagnostics);
    add_diagnostic!(ungrouped_select_variable::diagnostics);
    add_diagnostic!(invalid_projection_variable::diagnostics);
    add_diagnostic!(same_subject::diagnostics);
    add_diagnostic!(syntax_error::diagnostics);
    add_diagnostic!(duplicate_prefix_declaration::diagnostics);
    add_diagnostic!(groupby_star_selection::diagnostics);

    if client_support_workspace_edits(&server) {
        auto_fix_prefixes(&mut server, &request, &diagnostic_accu);
    }

    server.send_message(DiagnosticResponse::new(request.get_id(), diagnostic_accu))
}
