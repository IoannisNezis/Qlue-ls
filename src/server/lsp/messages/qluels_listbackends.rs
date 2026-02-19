use crate::server::{
    configuration::BackendConfiguration,
    lsp::{
        LspMessage,
        rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct ListBackendsRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
}

impl ListBackendsRequest {
    pub fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

impl LspMessage for ListBackendsRequest {}

#[derive(Debug, Serialize, PartialEq)]
pub struct ListBackendsResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    pub result: Vec<ListBackendsItem>,
}

impl ListBackendsResponse {
    pub(crate) fn new(id: &RequestId, backends: Vec<&BackendConfiguration>) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: backends
                .iter()
                .map(|backend| ListBackendsItem {
                    name: backend.name.clone(),
                    url: backend.url.clone(),
                    default: backend.default,
                })
                .collect(),
        }
    }
}

impl LspMessage for ListBackendsResponse {}

#[derive(Debug, Serialize, PartialEq)]
pub struct ListBackendsItem {
    pub name: String,
    pub url: String,
    pub default: bool,
}
