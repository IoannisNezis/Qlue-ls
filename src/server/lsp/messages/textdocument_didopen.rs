use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    LspMessage, rpc::NotificationMessageBase, textdocument::TextDocumentItem,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DidOpenTextDocumentNotification {
    #[serde(flatten)]
    base: NotificationMessageBase,
    pub params: DidOpenTextDocumentPrams,
}

impl LspMessage for DidOpenTextDocumentNotification {}

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
