use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    LspMessage, RequestMarker, ResponseMarker,
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct PingBackendRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: PingBackendParams,
}

impl LspMessage for PingBackendRequest {
    type Kind = RequestMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/pingBackend")
    }

    fn id(&self) -> Option<&RequestId> {
        Some(&self.base.id)
    }
}

impl PingBackendRequest {
    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PingBackendParams {
    pub backend_name: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct PingBackendResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    pub result: PingBackendResult,
}

impl LspMessage for PingBackendResponse {
    type Kind = ResponseMarker;

    fn method(&self) -> Option<&str> {
        None
    }

    fn id(&self) -> Option<&RequestId> {
        self.base.request_id()
    }
}

impl PingBackendResponse {
    pub fn new(id: &RequestId, available: bool) -> Self {
        PingBackendResponse {
            base: ResponseMessageBase::success(id),
            result: PingBackendResult { available },
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct PingBackendResult {
    pub available: bool,
}
