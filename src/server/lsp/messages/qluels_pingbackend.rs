use serde::{Deserialize, Serialize};

use crate::server::lsp::rpc::{RequestId, RequestMessageBase, ResponseMessageBase};

#[derive(Debug, Deserialize, PartialEq)]
pub struct PingBackendRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: PingBackendParams,
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

impl PingBackendResponse {
    pub fn new(id: &RequestId, availible: bool) -> Self {
        PingBackendResponse {
            base: ResponseMessageBase::success(id),
            result: PingBackendResult { availible },
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct PingBackendResult {
    pub availible: bool,
}
