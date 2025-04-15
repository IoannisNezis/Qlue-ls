use serde::Serialize;

use crate::server::lsp::{
    base_types::LSPAny,
    rpc::{RequestId, ResponseMessageBase},
};

#[derive(Debug, Serialize, PartialEq)]
pub struct ShutdownResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<LSPAny>,
}

impl ShutdownResponse {
    pub fn new(id: &RequestId) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: Some(LSPAny::Null),
        }
    }
}
