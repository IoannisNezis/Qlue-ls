mod query;
mod update;
mod utils;

use crate::server::{
    Server,
    lsp::{
        ExecuteOperationRequest,
        errors::{ErrorCode, LSPError},
    },
    message_handler::execute::{
        query::handle_execute_query_request, update::handle_execute_update_request,
    },
};
use futures::lock::Mutex;
use ll_sparql_parser::{TopEntryPoint, guess_operation_type};
use std::rc::Rc;

pub(super) async fn handle_execute_request(
    server_rc: Rc<Mutex<Server>>,
    request: ExecuteOperationRequest,
) -> Result<(), LSPError> {
    let (query, url, engine) = {
        let server = server_rc.lock().await;
        let text = server
            .state
            .get_document(&request.params.text_document.uri)?
            .text
            .clone();
        let service = server.state.get_default_backend().ok_or(LSPError::new(
            ErrorCode::InvalidRequest,
            "Can not execute operation, no SPARQL endpoint was specified",
        ))?;
        (text, service.url.clone(), service.engine.clone())
    };

    match guess_operation_type(&query) {
        Some(TopEntryPoint::QueryUnit) => {
            handle_execute_query_request(server_rc, request, url, query, engine).await
        }
        Some(TopEntryPoint::UpdateUnit) => {
            handle_execute_update_request(server_rc, request, url, query).await
        }
        None => {
            log::warn!("Could not determine operation type.\nFalling back to Query.");
            handle_execute_query_request(server_rc, request, url, query, engine).await
        }
    }
}
