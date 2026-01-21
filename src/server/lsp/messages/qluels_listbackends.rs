use crate::server::lsp::{
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    BackendService, LspMessage,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct ListBackendsRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
}

impl ListBackendsRequest {
    pub fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

impl LspMessage for ListBackendsRequest {}

#[derive(Debug, Serialize, PartialEq)]
pub struct ListBackendsResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    pub result: Vec<BackendService>,
}
impl ListBackendsResponse {
    pub(crate) fn new(id: &RequestId, backend: Vec<BackendService>) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: backend,
        }
    }
}

impl LspMessage for ListBackendsResponse {}
