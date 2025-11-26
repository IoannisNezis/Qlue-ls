use crate::server::lsp::{
    BackendService, LspMessage, RequestMarker, ResponseMarker,
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct GetBackendRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
}

impl GetBackendRequest {
    pub fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

impl LspMessage for GetBackendRequest {
    type Kind = RequestMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/getBackend")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        Some(&self.base.id)
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct GetBackendResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    pub result: Option<BackendService>,
}
impl GetBackendResponse {
    pub(crate) fn new(id: &RequestId, backend: Option<BackendService>) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: backend,
        }
    }
}

impl LspMessage for GetBackendResponse {
    type Kind = ResponseMarker;

    fn method(&self) -> Option<&str> {
        None
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        None
    }
}
