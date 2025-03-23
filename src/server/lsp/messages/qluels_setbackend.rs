use serde::{Deserialize, Serialize};

use crate::server::lsp::rpc::{RequestId, RequestMessageBase, ResponseMessageBase};

#[derive(Debug, Deserialize, PartialEq)]
pub struct SetBackendRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: SetBackendParams,
}

impl SetBackendRequest {
    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetBackendParams {
    pub name: String,
    pub url: String,
    pub health_check_url: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct SetBackendResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    pub result: SetBackendResult,
}

impl SetBackendResponse {
    pub fn new(id: &RequestId, availible: bool) -> Self {
        SetBackendResponse {
            base: ResponseMessageBase::success(id),
            result: SetBackendResult { availible },
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct SetBackendResult {
    pub availible: bool,
}
