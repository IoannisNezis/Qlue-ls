use serde::Deserialize;

use crate::server::lsp::rpc::NotificationMessageBase;

#[derive(Debug, Deserialize, PartialEq)]
pub struct UpdateDefaultBackendNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: UpdateDefaultBackendParams,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDefaultBackendParams {
    pub backend_name: String,
}
