use crate::server::{
    configuration::BackendConfiguration,
    lsp::{LspMessage, rpc::NotificationMessageBase},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct AddBackendNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: BackendConfiguration,
}

impl LspMessage for AddBackendNotification {}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum SparqlEngine {
    QLever,
    GraphDB,
    Virtuoso,
    MillenniumDB,
    Blazegraph,
    Jena,
}
