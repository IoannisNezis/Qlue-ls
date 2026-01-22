use crate::{
    server::{
        Server,
        configuration::RequestMethod,
        lsp::{
            ExecuteOperationErrorData, ExecuteOperationRequest, ExecuteOperationResponse,
            ExecuteOperationResponseResult, ExecuteQueryResponseResult, SparqlEngine,
            errors::LSPError,
        },
        message_handler::execute::utils::get_timestamp,
        sparql_operations::{SparqlRequestError, execute_construct_query, execute_query},
    },
    sparql::results::RDFTerm,
};
use futures::lock::Mutex;
use ll_sparql_parser::{QueryType, guess_query_type};
use std::rc::Rc;

pub(super) async fn handle_execute_query_request(
    server_rc: Rc<Mutex<Server>>,
    request: ExecuteOperationRequest,
    url: String,
    query: String,
    engine: Option<SparqlEngine>,
) -> Result<(), LSPError> {
    match guess_query_type(&query) {
        Some(QueryType::SelectQuery | QueryType::DescribeQuery | QueryType::AskQuery) => {
            handle_normal_query(server_rc, request, url, query, engine).await
        }
        Some(QueryType::ConstructQuery) => {
            handle_construct_query(server_rc, request, url, query, engine).await
        }
        None => {
            log::warn!("Cound not determine Query-type, falling back to SelectQuery");
            handle_normal_query(server_rc, request, url, query, engine).await
        }
    }
}

async fn handle_normal_query(
    server_rc: Rc<Mutex<Server>>,
    request: ExecuteOperationRequest,
    url: String,
    query: String,
    engine: Option<SparqlEngine>,
) -> Result<(), LSPError> {
    let start_time = get_timestamp();
    let query_result = match execute_query(
        server_rc.clone(),
        url,
        query,
        request.params.query_id.as_ref().map(|s| s.as_ref()),
        engine,
        None,
        RequestMethod::POST,
        request.params.max_result_size,
        request.params.result_offset.unwrap_or(0),
        request.params.lazy.unwrap_or(false),
    )
    .await
    {
        Ok(res) => res,

        #[cfg(target_arch = "wasm32")]
        Err(SparqlRequestError::QLeverException(exception)) => {
            return server_rc
                .lock()
                .await
                .send_message(ExecuteOperationResponse::error(
                    request.get_id(),
                    ExecuteOperationErrorData::QLeverException(exception),
                ));
        }
        Err(SparqlRequestError::Connection(error)) => {
            return server_rc
                .lock()
                .await
                .send_message(ExecuteOperationResponse::error(
                    request.get_id(),
                    ExecuteOperationErrorData::Connection(error),
                ));
        }
        Err(SparqlRequestError::_Canceled(error)) => {
            log::info!("Sending cancel error");
            return server_rc
                .lock()
                .await
                .send_message(ExecuteOperationResponse::error(
                    request.get_id(),
                    ExecuteOperationErrorData::Canceled(error),
                ));
        }
        Err(_err) => {
            return server_rc
                .lock()
                .await
                .send_message(ExecuteOperationResponse::error(
                    request.get_id(),
                    ExecuteOperationErrorData::Unknown,
                ));
        }
    };
    let stop_time = get_timestamp();
    let duration = stop_time - start_time;
    if request.params.lazy.unwrap_or(false) {
        server_rc
            .lock()
            .await
            .send_message(ExecuteOperationResponse::success(
                request.get_id(),
                ExecuteOperationResponseResult::QueryResult(ExecuteQueryResponseResult {
                    time_ms: duration,
                    result: None,
                }),
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
        server.send_message(ExecuteOperationResponse::success(
            request.get_id(),
            ExecuteOperationResponseResult::QueryResult(ExecuteQueryResponseResult {
                time_ms: duration,
                result: Some(query_result),
            }),
        ))
    }
}

async fn handle_construct_query(
    server_rc: Rc<Mutex<Server>>,
    request: ExecuteOperationRequest,
    url: String,
    query: String,
    engine: Option<SparqlEngine>,
) -> Result<(), LSPError> {
    let result = match execute_construct_query(
        server_rc.clone(),
        &url,
        &query,
        request.params.query_id.as_ref().map(|s| s.as_ref()),
        engine,
        request.params.lazy.unwrap_or(false),
    )
    .await
    {
        Ok(res) => res,
        #[cfg(target_arch = "wasm32")]
        Err(SparqlRequestError::QLeverException(exception)) => {
            return server_rc
                .lock()
                .await
                .send_message(ExecuteOperationResponse::error(
                    request.get_id(),
                    ExecuteOperationErrorData::QLeverException(exception),
                ));
        }
        Err(SparqlRequestError::Connection(error)) => {
            return server_rc
                .lock()
                .await
                .send_message(ExecuteOperationResponse::error(
                    request.get_id(),
                    ExecuteOperationErrorData::Connection(error),
                ));
        }
        Err(SparqlRequestError::_Canceled(error)) => {
            log::info!("Sending cancel error");
            return server_rc
                .lock()
                .await
                .send_message(ExecuteOperationResponse::error(
                    request.get_id(),
                    ExecuteOperationErrorData::Canceled(error),
                ));
        }
        Err(_err) => {
            return server_rc
                .lock()
                .await
                .send_message(ExecuteOperationResponse::error(
                    request.get_id(),
                    ExecuteOperationErrorData::Unknown,
                ));
        }
    };

    server_rc
        .lock()
        .await
        .send_message(ExecuteOperationResponse::success(
            request.get_id(),
            ExecuteOperationResponseResult::QueryResult(ExecuteQueryResponseResult {
                time_ms: 0,
                result: result,
            }),
        ))
}
