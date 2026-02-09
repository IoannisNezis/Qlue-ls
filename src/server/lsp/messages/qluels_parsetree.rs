use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    LspMessage,
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::{Range, TextDocumentIdentifier},
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct ParseTreeRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: ParseTreeParams,
}

impl LspMessage for ParseTreeRequest {}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParseTreeParams {
    pub text_document: TextDocumentIdentifier,
    pub skip_trivia: Option<bool>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ParseTreeResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: ParseTreeElement,
}

impl LspMessage for ParseTreeResponse {}

impl ParseTreeResponse {
    pub fn new(id: &RequestId, result: ParseTreeElement) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ParseTreeElement {
    Node {
        kind: String,
        range: Range,
        children: Vec<ParseTreeElement>,
    },
    Token {
        kind: String,
        range: Range,
        text: String,
    },
}
