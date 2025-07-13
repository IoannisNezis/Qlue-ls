use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    rpc::NotificationMessage, textdocument::TextDocumentItem, LspMessage, NotificationMarker,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DidOpenTextDocumentNotification {
    #[serde(flatten)]
    base: NotificationMessage,
    pub params: DidOpenTextDocumentPrams,
}

impl LspMessage for DidOpenTextDocumentNotification {
    type Kind = NotificationMarker;

    fn method(&self) -> Option<&str> {
        Some("textDocument/didOpen")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        None
    }
}

impl DidOpenTextDocumentNotification {
    pub fn get_text_document(self) -> TextDocumentItem {
        self.params.text_document
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidOpenTextDocumentPrams {
    pub text_document: TextDocumentItem,
}
