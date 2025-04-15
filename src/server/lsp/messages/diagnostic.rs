use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::server::lsp::{base_types::LSPAny, textdocument::Range};

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnostic
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Diagnostic {
    /**
     * The range at which the message applies.
     */
    pub(crate) range: Range,
    /**
     * The diagnostic's severity. To avoid interpretation mismatches when a
     * server is used with different clients it is highly recommended that
     * servers always provide a severity value. If omitted, it’s recommended
     * for the client to interpret it as an Error severity.
     */
    pub(crate) severity: DiagnosticSeverity,

    /**
     * The diagnostic's code, which might appear in the user interface.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) code: Option<DiagnosticCode>,
    // codeDescription: CodeDescription
    /**
     * A human-readable string describing the source of this
     * diagnostic, e.g. 'typescript' or 'super lint'.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /**
     * The diagnostic's message.
     */
    pub message: String,
    // tags
    // relatedInformation
    /**
     * A data entry field that is preserved between a
     * `textDocument/publishDiagnostics` notification and
     * `textDocument/codeAction` request.
     *
     * @since 3.16.0
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<LSPAny>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum DiagnosticCode {
    String(String),
    Integer(i32),
}

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnosticSeverity
#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}
