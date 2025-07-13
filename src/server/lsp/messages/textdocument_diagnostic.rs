use serde::{Deserialize, Serialize};

use crate::server::lsp::rpc::{RequestId, RequestMessageBase, ResponseMessageBase};
use crate::server::lsp::textdocument::TextDocumentIdentifier;
use crate::server::lsp::{LspMessage, RequestMarker, ResponseMarker};

use super::diagnostic::Diagnostic;

#[derive(Debug, Deserialize, PartialEq)]
pub struct DiagnosticRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: DocumentDiagnosticParams,
}

impl LspMessage for DiagnosticRequest {
    type Kind = RequestMarker;

    fn method(&self) -> Option<&str> {
        Some("textDocument/diagnostic")
    }

    fn id(&self) -> Option<&RequestId> {
        Some(&self.base.id)
    }
}

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

impl LspMessage for DiagnosticResponse {
    type Kind = ResponseMarker;

    fn method(&self) -> Option<&str> {
        None
    }

    fn id(&self) -> Option<&RequestId> {
        self.base.request_id()
    }
}

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
