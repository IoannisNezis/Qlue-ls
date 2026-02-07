use crate::server::{
    configuration::BackendConfiguration,
    lsp::{
        LspMessage,
        errors::{ErrorCode, LSPError},
        rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct GetBackendRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
}

impl GetBackendRequest {
    pub fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

impl LspMessage for GetBackendRequest {}

#[derive(Debug, Serialize, PartialEq)]
pub struct GetBackendResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    pub result: Option<BackendConfiguration>,
    pub error: Option<GetBackendError>,
}

impl GetBackendResponse {
    pub(crate) fn new(id: &RequestId, backend: Option<&BackendConfiguration>) -> Self {
        if let Some(backend) = backend {
            Self {
                base: ResponseMessageBase::success(id),
                result: Some(backend.clone()),
                error: None,
            }
        } else {
            Self {
                base: ResponseMessageBase::success(id),
                result: None,
                error: Some(LSPError {
                    code: ErrorCode::InvalidParams,
                    message: "No default backend is configured.".to_string(),
                    data: None,
                }),
            }
        }
    }
}

impl LspMessage for GetBackendResponse {}

pub type GetBackendError = LSPError;
