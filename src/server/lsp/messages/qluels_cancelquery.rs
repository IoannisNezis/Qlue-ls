use serde::Deserialize;

use crate::server::lsp::{
    LspMessage, NotificationMarker,
    rpc::{RequestId, RequestMessageBase},
};

#[derive(Debug, Deserialize)]
pub struct CancelQueryNotification {
    #[serde(flatten)]
    base: RequestMessageBase,
    pub params: CancelQueryParams,
}

impl CancelQueryNotification {
    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

impl LspMessage for CancelQueryNotification {
    type Kind = NotificationMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/cancelQuery")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        None
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelQueryParams {
    pub query_id: String,
}
