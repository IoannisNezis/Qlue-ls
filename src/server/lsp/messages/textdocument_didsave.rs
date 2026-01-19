use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    LspMessage, rpc::NotificationMessageBase, textdocument::TextDocumentIdentifier,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DidSaveTextDocumentNotification {
    #[serde(flatten)]
    base: NotificationMessageBase,
    pub params: DidSaveTextDocumentParams,
}

impl LspMessage for DidSaveTextDocumentNotification {}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidSaveTextDocumentParams {
    pub text_document: TextDocumentIdentifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}
