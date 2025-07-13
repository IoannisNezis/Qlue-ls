use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::server::{
    configuration::RequestMethod,
    lsp::{rpc::NotificationMessageBase, LspMessage, NotificationMarker},
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct AddBackendNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: SetBackendParams,
}

impl LspMessage for AddBackendNotification {
    type Kind = NotificationMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/addBackend")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        None
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetBackendParams {
    pub backend: Backend,
    pub request_method: Option<RequestMethod>,
    pub default: bool,
    pub prefix_map: Option<HashMap<String, String>>,
    pub queries: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Backend {
    pub name: String,
    pub url: String,
    pub health_check_url: Option<String>,
}
