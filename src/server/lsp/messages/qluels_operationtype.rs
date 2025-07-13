use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::TextDocumentIdentifier,
    LspMessage, RequestMarker, ResponseMarker,
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct IdentifyOperationTypeRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: IdentifyOperationTypeParams,
}

impl LspMessage for IdentifyOperationTypeRequest {
    type Kind = RequestMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/identifyOperationType")
    }

    fn id(&self) -> Option<&RequestId> {
        Some(&self.base.id)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IdentifyOperationTypeParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IdentifyOperationTypeResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: DetermineOperationTypeResult,
}

impl LspMessage for IdentifyOperationTypeResponse {
    type Kind = ResponseMarker;

    fn method(&self) -> Option<&str> {
        None
    }

    fn id(&self) -> Option<&RequestId> {
        self.base.request_id()
    }
}

impl IdentifyOperationTypeResponse {
    pub fn new(id: RequestId, operation_type: OperationType) -> Self {
        Self {
            base: ResponseMessageBase::success(&id),
            result: DetermineOperationTypeResult { operation_type },
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DetermineOperationTypeResult {
    operation_type: OperationType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperationType {
    Query,
    Update,
    Unknown,
}

impl Serialize for OperationType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            OperationType::Query => "Query",
            OperationType::Update => "Update",
            OperationType::Unknown => "Unknown",
        })
    }
}
