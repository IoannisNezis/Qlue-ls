use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::{
    ast::{AstNode, QueryUnit, ServiceGraphPattern},
    SyntaxToken,
};

use crate::server::{
    lsp::{errors::LSPError, Backend, SetTraceNotification},
    Server,
};

pub(super) async fn handle_set_trace_notification(
    server: Rc<Mutex<Server>>,
    set_trace_notification: SetTraceNotification,
) -> Result<(), LSPError> {
    log::info!("Trace set to: {:?}", set_trace_notification.params.value);
    server.lock().await.state.trace_value = set_trace_notification.params.value;
    Ok(())
}

/// Resolve which Backend to use at given token.
/// Currently only works for Query operations.
pub(super) fn resolve_backend(
    server: &Server,
    query_unit: &QueryUnit,
    token: &SyntaxToken,
) -> Option<Backend> {
    token
        .parent_ancestors()
        .find_map(ServiceGraphPattern::cast)
        .and_then(|service| {
            service
                .iri()
                .and_then(|iri| iri.raw_iri())
                .or(service.iri().and_then(|iri| {
                    iri.prefixed_name().and_then(|prefixed_name| {
                        query_unit.prologue().and_then(|prologue| {
                            prologue
                                .prefix_declarations()
                                .iter()
                                .find_map(|prefix_declaration| {
                                    prefix_declaration
                                        .prefix()
                                        .is_some_and(|prefix| prefix == prefixed_name.prefix())
                                        .then_some(prefix_declaration.raw_uri_prefix())
                                        .flatten()
                                })
                        })
                    })
                }))
        })
        .and_then(|iri_string| server.state.get_backend_name_by_url(&iri_string))
        .and_then(|backend_name| server.state.get_backend(&backend_name).cloned())
        .or(server.state.get_default_backend().cloned())
}
