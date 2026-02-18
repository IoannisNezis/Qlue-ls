use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::{Position, TextDocumentIdentifier, TextEdit},
    FormattingOptions, LspMessage,
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct OnTypeFormattingRequest {
    #[serde(flatten)]
    base: RequestMessageBase,
    params: DocumentOnTypeFormattingParams,
}

impl LspMessage for OnTypeFormattingRequest {}

impl OnTypeFormattingRequest {
    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }

    pub fn get_document_uri(&self) -> &String {
        &self.params.text_document.uri
    }

    pub fn get_position(&self) -> &Position {
        &self.params.position
    }

    pub fn get_char(&self) -> &str {
        &self.params.ch
    }

    pub(crate) fn get_options(&self) -> &FormattingOptions {
        &self.params.options
    }
}

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#documentOnTypeFormattingParams
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DocumentOnTypeFormattingParams {
    text_document: TextDocumentIdentifier,
    position: Position,
    /// The character that has been typed that triggered the formatting on type request.
    ch: String,
    options: FormattingOptions,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct OnTypeFormattingResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: Option<Vec<TextEdit>>,
}

impl LspMessage for OnTypeFormattingResponse {}

impl OnTypeFormattingResponse {
    pub(crate) fn new(id: &RequestId, text_edits: Vec<TextEdit>) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: Some(text_edits),
        }
    }

    pub(crate) fn null(id: &RequestId) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: None,
        }
    }
}
