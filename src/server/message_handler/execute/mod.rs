mod query;
mod utils;
use crate::server::{
    Server,
    lsp::{
        ExecuteQueryRequest,
        errors::{ErrorCode, LSPError},
    },
    message_handler::execute::query::handle_execute_query_request,
};
use futures::lock::Mutex;
use ll_sparql_parser::{TopEntryPoint, guess_operation_type};
use std::rc::Rc;

pub(super) async fn handle_execute_request(
    server_rc: Rc<Mutex<Server>>,
    request: ExecuteQueryRequest,
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
            handle_execute_query_request(server_rc, request, url, query, engine)
        }
        Some(TopEntryPoint::UpdateUnit) => {
            // TODO: support update
            todo!()
        }
        None => {
            log::warn!("Could not determine operation type.\nFalling back to Query.");
            handle_execute_query_request(server_rc, request, url, query, engine)
        }
    }
    .await
}
