use crate::server::lsp::{
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    BackendService, LspMessage, RequestMarker, ResponseMarker,
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

impl LspMessage for ListBackendsRequest {
    type Kind = RequestMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/listBackends")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        Some(&self.base.id)
    }
}

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

impl LspMessage for ListBackendsResponse {
    type Kind = ResponseMarker;

    fn method(&self) -> Option<&str> {
        None
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        None
    }
}
