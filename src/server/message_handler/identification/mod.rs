use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::{TopEntryPoint, guess_operation_type};

use crate::server::{
    Server,
    lsp::{
        IdentifyOperationTypeRequest, IdentifyOperationTypeResponse, OperationType,
        errors::LSPError,
    },
};

pub(super) async fn handle_identify_request(
    server_rc: Rc<Mutex<Server>>,
    request: IdentifyOperationTypeRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let document = server
        .state
        .get_document(&request.params.text_document.uri)?;

    let operation_type = match guess_operation_type(&document.text) {
        Some(TopEntryPoint::QueryUnit) => OperationType::Query,
        Some(TopEntryPoint::UpdateUnit) => OperationType::Update,
        None => OperationType::Unknown,
    };

    server.send_message(IdentifyOperationTypeResponse::new(
        request.base.id,
        operation_type,
    ))
}
