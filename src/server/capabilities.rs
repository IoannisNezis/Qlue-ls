//! LSP capability declaration.
//!
//! This module defines what features the server advertises to clients during
//! the `initialize` handshake. Capabilities tell the client which LSP methods
//! the server supports.
//!
//! # Key Functions
//!
//! - [`create_capabilities`]: Builds the `ServerCapabilities` struct sent to clients
//!
//! # Supported Features
//!
//! - Incremental text document sync
//! - Hover, code actions, diagnostics
//! - Completions (triggered by `?` and space)
//! - Document formatting
//! - Folding ranges
//!
//! # Related Modules
//!
//! - [`super::lsp::capabilities`]: Type definitions for capability structs
//! - [`super::message_handler::lifecycle`]: Sends capabilities in `initialize` response

use super::lsp::capabilities::{
    CompletionOptions, DiagnosticOptions, DocumentFormattingOptions, ExecuteCommandOptions,
    ServerCapabilities, TextDocumentSyncKind, WorkDoneProgressOptions,
};

pub(super) fn create_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: TextDocumentSyncKind::Incremental,
        hover_provider: true,
        code_action_provider: true,
        execute_command_provider: ExecuteCommandOptions {
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: true,
            },
            commands: vec![String::from("publish diagnostics")],
        },
        diagnostic_provider: DiagnosticOptions {
            identifier: "qlue-ls".to_string(),
            inter_file_dependencies: false,
            workspace_diagnostics: false,
        },
        completion_provider: CompletionOptions {
            trigger_characters: vec!["?".to_string(), " ".to_string()],
        },
        document_formatting_provider: DocumentFormattingOptions {},
        folding_range_provider: true,
    }
}
