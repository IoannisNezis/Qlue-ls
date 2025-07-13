use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::Position,
    LspMessage, RequestMarker, ResponseMarker,
};

use super::utils::TextDocumentPositionParams;

#[derive(Debug, Deserialize, PartialEq)]
pub struct JumpRequest {
    #[serde(flatten)]
    base: RequestMessageBase,
    pub params: JumpParams,
}

impl LspMessage for JumpRequest {
    type Kind = RequestMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/jump")
    }

    fn id(&self) -> Option<&RequestId> {
        Some(&self.base.id)
    }
}

impl JumpRequest {
    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct JumpParams {
    #[serde(flatten)]
    pub base: TextDocumentPositionParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<bool>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct JumpResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: Option<JumpResult>,
}

impl LspMessage for JumpResponse {
    type Kind = ResponseMarker;

    fn method(&self) -> Option<&str> {
        None
    }

    fn id(&self) -> Option<&RequestId> {
        self.base.request_id()
    }
}

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JumpResult {
    pub position: Position,
    pub insert_before: Option<String>,
    pub insert_after: Option<String>,
}

impl JumpResult {
    pub fn new(
        position: Position,
        insert_before: Option<&str>,
        insert_after: Option<&str>,
    ) -> Self {
        Self {
            position,
            insert_before: insert_before.map(|s| s.to_string()),
            insert_after: insert_after.map(|s| s.to_string()),
        }
    }
}

impl JumpResponse {
    pub(crate) fn new(id: &RequestId, result: Option<JumpResult>) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result,
        }
    }
}
