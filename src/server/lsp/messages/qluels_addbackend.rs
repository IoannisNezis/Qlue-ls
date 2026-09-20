use crate::server::{
    configuration::BackendConfiguration,
    lsp::{
        LspMessage,
        base_types::LSPAny,
        rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct AddBackendRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: BackendConfiguration,
}

impl AddBackendRequest {
    pub fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

impl LspMessage for AddBackendRequest {}

#[derive(Debug, Serialize, PartialEq)]
pub struct AddBackendResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<LSPAny>,
}

impl AddBackendResponse {
    pub fn new(id: &RequestId) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: Some(LSPAny::Null),
        }
    }
}

impl LspMessage for AddBackendResponse {}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum SparqlEngine {
    QLever,
    GraphDB,
    Virtuoso,
    MillenniumDB,
    Blazegraph,
    Jena,
}
