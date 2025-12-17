use std::rc::Rc;

use futures::lock::Mutex;

use crate::{
    server::{
        Server,
        configuration::RequestMethod,
        lsp::{
            ExecuteQueryErrorData, ExecuteQueryRequest, ExecuteQueryResponse,
            ExecuteQueryResponseResult,
            errors::{ErrorCode, LSPError},
        },
        sparql_operations::{SparqlRequestError, Window, fetch_sparql_result},
    },
    sparql::results::RDFTerm,
};

pub(super) async fn handle_execute_query_request(
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
            "Can not execute query, no SPARQL endpoint was specified",
        ))?;
        (text, service.url.clone(), service.engine.clone())
    };

    let start_time = get_timestamp();
    let query_result = match fetch_sparql_result(
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

#[cfg(not(target_arch = "wasm32"))]
fn get_timestamp() -> u128 {
    use std::time::Instant;
    Instant::now().elapsed().as_millis()
}

#[cfg(target_arch = "wasm32")]
fn get_timestamp() -> u128 {
    use wasm_bindgen::JsCast;
    use web_sys::WorkerGlobalScope;
    let worker_global: WorkerGlobalScope = js_sys::global().unchecked_into();
    worker_global
        .performance()
        .expect("performance should be available")
        .now() as u128
}
