use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    FormattingOptions, LspMessage,
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::{Position, TextEdit},
};

use super::utils::TextDocumentPositionParams;

#[derive(Debug, Deserialize, PartialEq)]
pub struct JumpRequest {
    #[serde(flatten)]
    base: RequestMessageBase,
    pub params: JumpParams,
}

impl LspMessage for JumpRequest {}

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
    // NOTE: The document is formatted before the jump target is computed.
    pub options: Option<FormattingOptions>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct JumpResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: Option<JumpResult>,
}

impl LspMessage for JumpResponse {}

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JumpResult {
    // NOTE: TextEdits against the document as it was in the request.
    // They contain the format edits and the placeholder insertions.
    pub edits: Vec<TextEdit>,
    // NOTE: Cursor position in the document AFTER all edits are applied.
    // `None` if no jump target was found.
    pub position: Option<Position>,
}

impl JumpResponse {
    pub(crate) fn new(id: &RequestId, result: Option<JumpResult>) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result,
        }
    }
}
