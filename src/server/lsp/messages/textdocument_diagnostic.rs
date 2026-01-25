use serde::{Deserialize, Serialize};

use crate::server::lsp::LspMessage;
use crate::server::lsp::rpc::{NotificationMessageBase, RequestId, RequestMessageBase, ResponseMessageBase};
use crate::server::lsp::textdocument::{DocumentUri, TextDocumentIdentifier};

use super::diagnostic::Diagnostic;

#[derive(Debug, Deserialize, PartialEq)]
pub struct DiagnosticRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: DocumentDiagnosticParams,
}

impl LspMessage for DiagnosticRequest {}

impl DiagnosticRequest {
    pub fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDiagnosticParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DiagnosticResponse {
    #[serde(flatten)]
    pub base: ResponseMessageBase,
    pub result: DocumentDiagnosticReport,
}

impl LspMessage for DiagnosticResponse {}

impl DiagnosticResponse {
    pub fn new(id: &RequestId, items: Vec<Diagnostic>) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: DocumentDiagnosticReport {
                kind: DocumentDiagnosticReportKind::Full,
                items,
            },
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DocumentDiagnosticReport {
    kind: DocumentDiagnosticReportKind,
    pub items: Vec<Diagnostic>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DocumentDiagnosticReportKind {
    Full,
    // Unchanged,
}

/// Notification sent from the server to the client to publish diagnostics.
///
/// See: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics
#[derive(Debug, Serialize, PartialEq)]
pub struct PublishDiagnosticsNotification {
    #[serde(flatten)]
    base: NotificationMessageBase,
    params: PublishDiagnosticsParams,
}

impl LspMessage for PublishDiagnosticsNotification {}

impl PublishDiagnosticsNotification {
    pub fn new(uri: DocumentUri, diagnostics: Vec<Diagnostic>, version: Option<u32>) -> Self {
        Self {
            base: NotificationMessageBase::new("textDocument/publishDiagnostics"),
            params: PublishDiagnosticsParams {
                uri,
                diagnostics,
                version,
            },
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct PublishDiagnosticsParams {
    /// The URI for which diagnostic information is reported.
    uri: DocumentUri,
    /// An array of diagnostic information items.
    diagnostics: Vec<Diagnostic>,
    /// Optional the version number of the document the diagnostics are published for.
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<u32>,
}
