pub mod invalid_projection_variable;
pub mod same_subject;
pub mod uncompacted_uri;
pub mod undeclared_prefix;
pub mod ungrouped_select_variable;
pub mod unused_prefix_declaration;

use crate::server::{
    lsp::{
        base_types::LSPAny, diagnostic::Diagnostic, errors::LSPError, DiagnosticRequest,
        DiagnosticResponse, WorkspaceEditRequest,
    },
    message_handler::code_action::declare_prefix,
    Server,
};
use futures::lock::Mutex;
use ll_sparql_parser::{
    ast::{AstNode, QueryUnit},
    parse,
};
use std::{
    collections::{HashMap, HashSet},
    convert::identity,
    rc::Rc,
};

use super::code_action::remove_prefix_declaration;

pub(super) async fn handle_diagnostic_request(
    server_rc: Rc<Mutex<Server>>,
    request: DiagnosticRequest,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
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
    add!(unused_prefix_declaration::diagnostics);
    add!(undeclared_prefix::diagnostics);
    add!(uncompacted_uri::diagnostics);
    add!(ungrouped_select_variable::diagnostics);
    add!(invalid_projection_variable::diagnostics);
    add!(same_subject::diagnostics);

    if client_support_workspace_edits(&server) {
        declare_and_undeclare_prefixes(&mut server, &request, &diagnostic_accu);
    }

    server.send_message(DiagnosticResponse::new(request.get_id(), diagnostic_accu))
}

fn declare_and_undeclare_prefixes(
    server: &mut Server,
    request: &DiagnosticRequest,
    diagnostics: &Vec<Diagnostic>,
) {
    let document_uri = request.params.text_document.uri.clone();
    let mut prefixes = HashSet::<&str>::new();
    let edits: Vec<_> = diagnostics
        .iter()
        .filter_map(|diagnostic| {
            if let Some(LSPAny::String(prefix)) = diagnostic.data.as_ref() {
                if prefixes.insert(prefix) {
                    match diagnostic.code.as_ref() {
                        Some(code)
                            if code == &*undeclared_prefix::CODE
                                && server.settings.prefixes.as_ref().is_some_and(|prefixes| {
                                    prefixes.add_missing.is_some_and(identity)
                                }) =>
                        {
                            declare_prefix(&server, &document_uri, diagnostic.clone())
                        }
                        Some(code)
                            if code == &*unused_prefix_declaration::CODE
                                && server.settings.prefixes.as_ref().is_some_and(|prefixes| {
                                    prefixes.remove_unused.is_some_and(identity)
                                }) =>
                        {
                            remove_prefix_declaration(server, &document_uri, diagnostic.clone())
                        }
                        _ => Ok(None),
                    }
                    .ok()
                    .flatten()
                    .and_then(|code_action| code_action.edit.changes)
                    .and_then(|mut changes| changes.remove(&document_uri))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .flatten()
        .collect();
    if !edits.is_empty() {
        let request_id = server.bump_request_id();
        if let Err(err) = server.send_message(WorkspaceEditRequest::new(
            request_id,
            HashMap::from_iter([(document_uri, edits)]),
        )) {
            log::error!("Sending \"workspace/applyEdit\" request failed:\n{:?}", err);
        }
    }
}

fn client_support_workspace_edits(server: &Server) -> bool {
    server
        .client_capabilities
        .as_ref()
        .is_some_and(|client_capabilities| {
            client_capabilities
                .workspace
                .as_ref()
                .and_then(|workspace_capabilities| workspace_capabilities.apply_edit)
                .is_some_and(|flag| flag)
                && client_capabilities
                    .workspace
                    .as_ref()
                    .and_then(|workspace_capabilities| {
                        workspace_capabilities.workspace_edit.as_ref()
                    })
                    .is_some_and(|capability| capability.document_changes.is_some_and(|flag| flag))
        })
}
