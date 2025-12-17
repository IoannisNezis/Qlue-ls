use crate::{
    server::{
        Server,
        configuration::RequestMethod,
        lsp::{
            ExecuteQueryErrorData, ExecuteQueryRequest, ExecuteQueryResponse,
            ExecuteQueryResponseResult, SparqlEngine, errors::LSPError,
        },
        message_handler::execute::utils::get_timestamp,
        sparql_operations::{SparqlRequestError, Window, execute_sparql_query},
    },
    sparql::results::RDFTerm,
};
use futures::lock::Mutex;
use std::rc::Rc;

pub(super) async fn execute_query(
    server_rc: Rc<Mutex<Server>>,
    request: ExecuteQueryRequest,
    url: String,
    query: String,
    engine: Option<SparqlEngine>,
) -> Result<(), LSPError> {
    let start_time = get_timestamp();
    let query_result = match execute_sparql_query(
        server_rc.clone(),
        &url,
        &query,
        request.params.query_id.as_ref().map(|s| s.as_ref()),
        engine,
        1000000,
        RequestMethod::POST,
        Some(Window::new(
            request.params.max_result_size.unwrap_or(100),
            request.params.result_offset.unwrap_or(0),
        )),
        request.params.lazy.unwrap_or(false),
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
    let stop_time = get_timestamp();
    let duration = stop_time - start_time;
    if request.params.lazy.unwrap_or(false) {
        server_rc
            .lock()
            .await
            .send_message(ExecuteQueryResponse::success(
                request.get_id(),
                ExecuteQueryResponseResult {
                    time_ms: duration,
                    result: None,
                },
            ))
    } else {
        let server = server_rc.lock().await;
        let mut query_result =
            query_result.expect("Non-lazy request should always return a result.");

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
            ExecuteQueryResponseResult {
                time_ms: duration,
                result: Some(query_result),
            },
        ))
    }
}
