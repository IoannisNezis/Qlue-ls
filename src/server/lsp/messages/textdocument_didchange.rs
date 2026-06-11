use core::fmt;

use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    LspMessage,
    rpc::NotificationMessageBase,
    textdocument::{Range, VersionedTextDocumentIdentifier},
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DidChangeTextDocumentNotification {
    #[serde(flatten)]
    base: NotificationMessageBase,
    pub params: DidChangeTextDocumentParams,
}

impl LspMessage for DidChangeTextDocumentNotification {}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeTextDocumentParams {
    pub text_document: VersionedTextDocumentIdentifier,
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocumentContentChangeEvent
// NOTE: `range` is absent when the event replaces the whole document.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TextDocumentContentChangeEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
    pub text: String,
}

impl fmt::Display for TextDocumentContentChangeEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.range {
            Some(range) => write!(f, "{:?}; [{}-{}]", self.text, range.start, range.end),
            None => write!(f, "{:?}; [full document]", self.text),
        }
    }
}
