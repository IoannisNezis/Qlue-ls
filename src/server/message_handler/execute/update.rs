use crate::server::{
    Server,
    lsp::{
        ExecuteOperationErrorData, ExecuteOperationRequest, ExecuteOperationResponse,
        ExecuteOperationResponseResult, errors::LSPError,
    },
    message_handler::execute::utils::get_timestamp,
    sparql_operations::{SparqlRequestError, execute_update},
};
use futures::lock::Mutex;
use std::rc::Rc;

pub(super) async fn handle_execute_update_request(
    server_rc: Rc<Mutex<Server>>,
    request: ExecuteOperationRequest,
    url: String,
    query: String,
) -> Result<(), LSPError> {
    let start_time = get_timestamp();
    let update_result = match execute_update(
        server_rc.clone(),
        &url,
        &query,
        request.params.query_id.as_ref().map(|s| s.as_ref()),
        request.params.access_token.as_ref().map(|s| s.as_ref()),
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
            return server_rc
                .lock()
                .await
                .send_message(ExecuteOperationResponse::error(
                    request.get_id(),
                    ExecuteOperationErrorData::Canceled(error),
                ));
        }
        Err(SparqlRequestError::Deserialization(error)) => {
            return server_rc
                .lock()
                .await
                .send_message(ExecuteOperationResponse::error(
                    request.get_id(),
                    ExecuteOperationErrorData::InvalidFormat {
                        query: query,
                        message: error,
                    },
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
    let _duration = stop_time - start_time;
    server_rc
        .lock()
        .await
        .send_message(ExecuteOperationResponse::success(
            request.get_id(),
            ExecuteOperationResponseResult::UpdateResult(update_result),
        ))
}
