use serde::Serialize;

use crate::server::lsp::{rpc::NotificationMessageBase, LspMessage, NotificationMarker};

use super::diagnostic::Diagnostic;

#[derive(Debug, Serialize, PartialEq)]
pub struct PublishDiagnosticsNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: PublishDiagnosticsParams,
}

impl LspMessage for PublishDiagnosticsNotification {
    type Kind = NotificationMarker;

    fn method(&self) -> Option<&str> {
        Some("textDocument/publishDiagnostics")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        None
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct PublishDiagnosticsParams {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}
