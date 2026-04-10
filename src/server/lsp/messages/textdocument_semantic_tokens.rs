use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    LspMessage,
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::{Range, TextDocumentIdentifier},
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct SemanticTokensFullRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: SemanticTokensParams,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticTokensParams {
    /// The text document.
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SemanticTokensRangeRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: SemanticTokensRangeParams,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticTokensRangeParams {
    /// The text document.
    pub text_document: TextDocumentIdentifier,

    /// The range the semantic tokens are requested for.
    pub range: Range,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct SemanticTokensFullResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: Option<SemanticTokens>,
}
impl SemanticTokensFullResponse {
    pub fn new(id: &RequestId, data: Vec<u32>) -> Self {
        SemanticTokensFullResponse {
            base: ResponseMessageBase::success(id),
            result: Some(SemanticTokens {
                result_id: None,
                data,
            }),
        }
    }
}

impl LspMessage for SemanticTokensFullResponse {}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SemanticTokens {
    /// An optional result id. If provided and clients support delta updating
    /// the client will include the result id in the next semantic token request.
    /// A server can then instead of computing all semantic tokens again simply
    /// send a delta.
    result_id: Option<String>,

    /// The actual tokens.
    data: Vec<u32>,
}
