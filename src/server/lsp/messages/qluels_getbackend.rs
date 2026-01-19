use crate::server::lsp::{
    BackendService, LspMessage,
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

impl LspMessage for GetBackendRequest {}

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

impl LspMessage for GetBackendResponse {}
