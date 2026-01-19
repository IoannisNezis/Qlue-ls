use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    LspMessage,
    base_types::LSPAny,
    rpc::{NotificationMessageBase, RequestId, RequestMessageBase, ResponseMessageBase},
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct ShutdownRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
}

impl LspMessage for ShutdownRequest {}

#[derive(Debug, Serialize, PartialEq)]
pub struct ShutdownResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<LSPAny>,
}

impl LspMessage for ShutdownResponse {}

impl ShutdownResponse {
    pub fn new(id: &RequestId) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: Some(LSPAny::Null),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ExitNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
}

impl LspMessage for ExitNotification {}
