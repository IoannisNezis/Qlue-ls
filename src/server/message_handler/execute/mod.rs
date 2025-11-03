use std::rc::Rc;

use futures::lock::Mutex;

use crate::{
    server::{
        configuration::RequestMethod,
        fetch::{fetch_sparql_result, SparqlRequestError, Window},
        lsp::{
            errors::{ErrorCode, LSPError},
            ExecuteQueryErrorData, ExecuteQueryRequest, ExecuteQueryResponse,
        },
        Server,
    },
    sparql::results::RDFTerm,
};

pub(super) async fn handle_execute_query_request(
    server_rc: Rc<Mutex<Server>>,
    request: ExecuteQueryRequest,
) -> Result<(), LSPError> {
    let (query, url) = {
        let server = server_rc.lock().await;
        let text = server
            .state
            .get_document(&request.params.text_document.uri)?
            .text
            .clone();
        let url = server
            .state
            .get_default_backend()
            .ok_or(LSPError::new(
                ErrorCode::InvalidRequest,
                "Can not execute query, no SPARQL endpoint was specified",
            ))?
            .url
            .clone();
        (text, url)
    };

    let mut query_result = match fetch_sparql_result(
        &url,
        &query,
        1000000,
        RequestMethod::POST,
        Some(Window::new(
            request.params.max_result_size.unwrap_or(100),
            request.params.result_offset.unwrap_or(0),
        )),
    )
    .await
    {
        Ok(res) => res,
        Err(SparqlRequestError::QLeverException(exception)) => {
            return server_rc
                .lock()
                .await
                .send_message(ExecuteQueryResponse::error(
                    request.get_id(),
                    ExecuteQueryErrorData::QLeverException(exception),
                ));
        }
        Err(SparqlRequestError::Connection(error)) => {
            return server_rc
                .lock()
                .await
                .send_message(ExecuteQueryResponse::error(
                    request.get_id(),
                    ExecuteQueryErrorData::Connection(error),
                ));
        }
        Err(_err) => {
            return server_rc
                .lock()
                .await
                .send_message(ExecuteQueryResponse::error(
                    request.get_id(),
                    ExecuteQueryErrorData::Unknown,
                ));
        }
    };

    let server = server_rc.lock().await;

    // NOTE: compress IRIs when possible.
    for binding in query_result.results.bindings.iter_mut() {
        for (_, rdf_term) in binding.iter_mut() {
            if let RDFTerm::Uri { value, curie } = rdf_term {
                *curie = server
                    .state
                    .get_default_converter()
                    .and_then(|converer| converer.compress(value).ok());
            }
        }
    }

    server.send_message(ExecuteQueryResponse::success(
        request.get_id(),
        query_result,
    ))
}
