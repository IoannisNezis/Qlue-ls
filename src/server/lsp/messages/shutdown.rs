use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    base_types::LSPAny,
    rpc::{NotificationMessage, RequestId, RequestMessageBase, ResponseMessageBase},
    LspMessage, NotificationMarker, RequestMarker, ResponseMarker,
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct ShutdownRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
}

impl LspMessage for ShutdownRequest {
    type Kind = RequestMarker;

    fn method(&self) -> Option<&str> {
        Some("shutdown")
    }

    fn id(&self) -> Option<&RequestId> {
        Some(&self.base.id)
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ShutdownResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<LSPAny>,
}

impl LspMessage for ShutdownResponse {
    type Kind = ResponseMarker;

    fn method(&self) -> Option<&str> {
        todo!()
    }

    fn id(&self) -> Option<&RequestId> {
        todo!()
    }
}

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
    pub base: NotificationMessage,
}

impl LspMessage for ExitNotification {
    type Kind = NotificationMarker;

    fn method(&self) -> Option<&str> {
        Some("exit")
    }

    fn id(&self) -> Option<&RequestId> {
        None
    }
}
