use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::server::{
    configuration::{CompletionTemplate, RequestMethod},
    lsp::{LspMessage, rpc::NotificationMessageBase},
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct AddBackendNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: AddBackendParams,
}

impl LspMessage for AddBackendNotification {}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AddBackendParams {
    pub service: BackendService,
    pub request_method: Option<RequestMethod>,
    pub default: bool,
    pub prefix_map: Option<HashMap<String, String>>,
    pub queries: Option<HashMap<CompletionTemplate, String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BackendService {
    pub name: String,
    pub url: String,
    pub health_check_url: Option<String>,
    pub engine: Option<SparqlEngine>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum SparqlEngine {
    QLever,
    GraphDB,
    Virtuoso,
    MillenniumDB,
    Blazegraph,
    Jena,
}
