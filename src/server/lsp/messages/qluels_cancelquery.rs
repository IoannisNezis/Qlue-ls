use serde::Deserialize;

use crate::server::lsp::{LspMessage, rpc::NotificationMessageBase};

#[derive(Debug, Deserialize)]
pub struct CancelQueryNotification {
    #[serde(flatten)]
    _base: NotificationMessageBase,
    pub params: CancelQueryParams,
}

impl LspMessage for CancelQueryNotification {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelQueryParams {
    pub query_id: String,
}
