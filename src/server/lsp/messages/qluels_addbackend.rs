use std::collections::HashMap;

use serde::Deserialize;

use crate::server::lsp::rpc::NotificationMessageBase;

#[derive(Debug, Deserialize, PartialEq)]
pub struct AddBackendNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: SetBackendParams,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetBackendParams {
    pub backend: Backend,
    pub default: bool,
    pub prefix_map: Option<HashMap<String, String>>,
    pub queries: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Backend {
    pub name: String,
    pub url: String,
    pub health_check_url: Option<String>,
}
