use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::server::lsp::{rpc::NotificationMessageBase, LspMessage, NotificationMarker};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ShowMessageNotification {
    #[serde(flatten)]
    base: NotificationMessageBase,
    params: ShowMessageParams,
}

impl LspMessage for ShowMessageNotification {
    type Kind = NotificationMarker;

    fn method(&self) -> Option<&str> {
        Some("window/showMessage")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        None
    }
}

#[allow(dead_code)]
impl ShowMessageNotification {
    pub fn new(message: &str, kind: MessageType) -> Self {
        Self {
            base: NotificationMessageBase::new("window/showMessage"),
            params: ShowMessageParams {
                kind,
                message: message.to_string(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ShowMessageParams {
    #[serde(rename = "type")]
    kind: MessageType,
    message: String,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum MessageType {
    Error = 1,
    Warning = 2,
    Info = 3,
    Log = 4,
    Debug = 5,
}
