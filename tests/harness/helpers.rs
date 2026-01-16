//! Helper methods for common LSP operations
//!
//! These methods provide a convenient API for interacting with the LSP server
//! during tests.

use super::TestClient;
use serde_json::{json, Value};

impl TestClient {
    // ========== Lifecycle Methods ==========

    /// Initialize the server with default capabilities.
    ///
    /// This sends both the `initialize` request and the `initialized` notification,
    /// which is required before most other LSP operations.
    pub async fn initialize(&self) -> u32 {
        let id = self
            .send_request(
                "initialize",
                json!({
                    "processId": null,
                    "capabilities": {
                        "textDocument": {
                            "completion": {
                                "completionItem": {
                                    "snippetSupport": true
                                }
                            }
                        }
                    },
                    "rootUri": "file:///test"
                }),
            )
            .await;

        // Send initialized notification after receiving response
        self.send_notification("initialized", json!({})).await;

        id
    }

    /// Initialize with custom parameters.
    pub async fn initialize_with(&self, params: Value) -> u32 {
        let id = self.send_request("initialize", params).await;
        self.send_notification("initialized", json!({})).await;
        id
    }

    /// Send shutdown request.
    pub async fn shutdown(&self) -> u32 {
        self.send_request("shutdown", json!(null)).await
    }

    /// Send exit notification.
    pub async fn exit(&self) {
        self.send_notification("exit", json!({})).await;
    }

    // ========== Document Methods ==========

    /// Open a document with the given URI and content.
    pub async fn open_document(&self, uri: &str, text: &str) {
        self.send_notification(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "sparql",
                    "version": 1,
                    "text": text
                }
            }),
        )
        .await;
    }

    /// Open a document and return a DocumentHandle for further operations.
    pub async fn open(&self, uri: &str, text: &str) -> DocumentHandle<'_> {
        self.open_document(uri, text).await;
        DocumentHandle {
            client: self,
            uri: uri.to_string(),
            version: 1,
        }
    }

    /// Change a document's content (full replacement).
    pub async fn change_document(&self, uri: &str, version: u32, text: &str) {
        self.send_notification(
            "textDocument/didChange",
            json!({
                "textDocument": {
                    "uri": uri,
                    "version": version
                },
                "contentChanges": [{ "text": text }]
            }),
        )
        .await;
    }

    /// Close a document.
    pub async fn close_document(&self, uri: &str) {
        self.send_notification(
            "textDocument/didClose",
            json!({
                "textDocument": { "uri": uri }
            }),
        )
        .await;
    }

    // ========== Feature Methods ==========

    /// Request formatting for a document.
    pub async fn format(&self, uri: &str) -> u32 {
        self.send_request(
            "textDocument/formatting",
            json!({
                "textDocument": { "uri": uri },
                "options": {
                    "tabSize": 2,
                    "insertSpaces": true
                }
            }),
        )
        .await
    }

    /// Request completion at a position.
    pub async fn complete(&self, uri: &str, line: u32, character: u32) -> u32 {
        self.send_request(
            "textDocument/completion",
            json!({
                "textDocument": { "uri": uri },
                "position": {
                    "line": line,
                    "character": character
                },
                "context": {
                    "triggerKind": 1
                }
            }),
        )
        .await
    }

    /// Request completion with trigger character.
    pub async fn complete_triggered(&self, uri: &str, line: u32, character: u32, trigger: &str) -> u32 {
        self.send_request(
            "textDocument/completion",
            json!({
                "textDocument": { "uri": uri },
                "position": {
                    "line": line,
                    "character": character
                },
                "context": {
                    "triggerKind": 2,
                    "triggerCharacter": trigger
                }
            }),
        )
        .await
    }

    /// Request hover information at a position.
    pub async fn hover(&self, uri: &str, line: u32, character: u32) -> u32 {
        self.send_request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": {
                    "line": line,
                    "character": character
                }
            }),
        )
        .await
    }

    /// Request diagnostics for a document.
    pub async fn diagnostics(&self, uri: &str) -> u32 {
        self.send_request(
            "textDocument/diagnostic",
            json!({
                "textDocument": { "uri": uri }
            }),
        )
        .await
    }

    /// Request code actions for a range.
    pub async fn code_actions(&self, uri: &str, start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> u32 {
        self.send_request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri },
                "range": {
                    "start": { "line": start_line, "character": start_char },
                    "end": { "line": end_line, "character": end_char }
                },
                "context": {
                    "diagnostics": []
                }
            }),
        )
        .await
    }

    /// Request folding ranges for a document.
    pub async fn folding_ranges(&self, uri: &str) -> u32 {
        self.send_request(
            "textDocument/foldingRange",
            json!({
                "textDocument": { "uri": uri }
            }),
        )
        .await
    }

    // ========== Custom Extension Methods (qlueLs/*) ==========

    /// Add a SPARQL backend.
    pub async fn add_backend(&self, name: &str, url: &str, default: bool) {
        self.send_notification(
            "qlueLs/addBackend",
            json!({
                "service": {
                    "name": name,
                    "url": url
                },
                "default": default,
                "prefixMap": {}
            }),
        )
        .await;
    }

    /// Add a backend with custom configuration.
    pub async fn add_backend_with(&self, config: Value) {
        self.send_notification("qlueLs/addBackend", config).await;
    }

    /// Get current backend information.
    pub async fn get_backend(&self) -> u32 {
        self.send_request("qlueLs/getBackend", json!({})).await
    }

    /// Ping a backend to check connectivity.
    pub async fn ping_backend(&self, name: Option<&str>) -> u32 {
        self.send_request(
            "qlueLs/pingBackend",
            json!({
                "backendName": name
            }),
        )
        .await
    }

    /// Change server settings.
    pub async fn change_settings(&self, settings: Value) {
        self.send_notification(
            "qlueLs/changeSettings",
            json!({
                "settings": settings
            }),
        )
        .await;
    }

    /// Identify the operation type (query vs update) of a document.
    pub async fn identify_operation_type(&self, uri: &str) -> u32 {
        self.send_request(
            "qlueLs/identifyOperationType",
            json!({
                "textDocument": { "uri": uri }
            }),
        )
        .await
    }

    /// Request jump navigation.
    pub async fn jump(&self, uri: &str, line: u32, character: u32, direction: &str) -> u32 {
        self.send_request(
            "qlueLs/jump",
            json!({
                "textDocument": { "uri": uri },
                "position": {
                    "line": line,
                    "character": character
                },
                "direction": direction
            }),
        )
        .await
    }
}

/// Handle for operations on an open document
pub struct DocumentHandle<'a> {
    client: &'a TestClient,
    pub uri: String,
    pub version: u32,
}

impl<'a> DocumentHandle<'a> {
    /// Change the document content
    pub async fn change(&mut self, text: &str) {
        self.version += 1;
        self.client.change_document(&self.uri, self.version, text).await;
    }

    /// Request formatting
    pub async fn format(&self) -> u32 {
        self.client.format(&self.uri).await
    }

    /// Request completion at position
    pub async fn complete(&self, line: u32, character: u32) -> u32 {
        self.client.complete(&self.uri, line, character).await
    }

    /// Request hover at position
    pub async fn hover(&self, line: u32, character: u32) -> u32 {
        self.client.hover(&self.uri, line, character).await
    }

    /// Request diagnostics
    pub async fn diagnostics(&self) -> u32 {
        self.client.diagnostics(&self.uri).await
    }

    /// Close this document
    pub async fn close(self) {
        self.client.close_document(&self.uri).await;
    }
}
