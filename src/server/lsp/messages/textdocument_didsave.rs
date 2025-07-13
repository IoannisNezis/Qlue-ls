use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    rpc::NotificationMessage, textdocument::TextDocumentItem, LspMessage, NotificationMarker,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DidSaveTextDocumentNotification {
    #[serde(flatten)]
    base: NotificationMessage,
    pub params: DidSaveTextDocumentParams,
}

impl LspMessage for DidSaveTextDocumentNotification {
    type Kind = NotificationMarker;

    fn method(&self) -> Option<&str> {
        Some("textDocument/didSave")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        None
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidSaveTextDocumentParams {
    pub text_document: TextDocumentItem,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}
