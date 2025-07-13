use serde::Deserialize;

use crate::server::lsp::{rpc::NotificationMessageBase, LspMessage, NotificationMarker};

#[derive(Debug, Deserialize, PartialEq)]
pub struct UpdateDefaultBackendNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: UpdateDefaultBackendParams,
}

impl LspMessage for UpdateDefaultBackendNotification {
    type Kind = NotificationMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/updateDefaultBackend")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        None
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDefaultBackendParams {
    pub backend_name: String,
}
