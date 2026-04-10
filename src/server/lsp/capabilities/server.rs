use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::{BoolOrEmpty, FullCapability};

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub text_document_sync: TextDocumentSyncKind,
    pub hover_provider: bool,
    pub completion_provider: CompletionOptions,
    pub document_formatting_provider: DocumentFormattingOptions,
    pub document_on_type_formatting_provider: DocumentOnTypeFormattingOptions,
    pub diagnostic_provider: DiagnosticOptions,
    pub code_action_provider: bool,
    pub execute_command_provider: ExecuteCommandOptions,
    pub folding_range_provider: bool,
    pub semantic_tokens_provider: SemanticTokensOptions,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ExecuteCommandOptions {
    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,
    pub commands: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkDoneProgressOptions {
    pub work_done_progress: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticOptions {
    pub identifier: String,
    pub inter_file_dependencies: bool,
    pub workspace_diagnostics: bool,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum TextDocumentSyncKind {
    None = 0,
    Full = 1,
    Incremental = 2,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletionOptions {
    // WARNING: This is not to spec, there are more optional options:
    // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#completionOptions
    pub trigger_characters: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct DocumentFormattingOptions {
    // WARNING: This could also inherit WorkDoneProgressOptions (not implemented yet).
}

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#documentOnTypeFormattingOptions
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentOnTypeFormattingOptions {
    /// A character on which formatting should be triggered, like `{`.
    pub first_trigger_character: String,
    /// More trigger characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more_trigger_character: Option<Vec<String>>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#semanticTokensOptions
///
/// @since 3.16.0
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SemanticTokensOptions {
    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,

    /// The legend used by the server.
    pub legend: SemanticTokensLegend,

    /// Server supports providing semantic tokens for a specific range of a
    /// document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<BoolOrEmpty>,

    /// Server supports providing semantic tokens for a full document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full: Option<FullCapability>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#semanticTokensLegend
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SemanticTokensLegend {
    /// The token types a server uses.
    pub token_types: Vec<SemanticTokenTypes>,
    /// The token modifiers a server uses.
    pub token_modifiers: Vec<SemanticTokenModifiers>,
}

/// Predefined semantic token types.
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#semanticTokenTypes
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Hash, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SemanticTokenTypes {
    Namespace,
    /// Represents a generic type. Acts as a fallback for types which
    /// can't be mapped to a specific type like class or enum.
    Type,
    Class,
    Enum,
    Interface,
    Struct,
    TypeParameter,
    Parameter,
    Variable,
    Property,
    EnumMember,
    Event,
    Function,
    Method,
    Macro,
    Keyword,
    Modifier,
    Comment,
    String,
    Number,
    Regexp,
    Operator,
    /// @since 3.17.0
    Decorator,
}

/// Predefined semantic token modifiers.
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#semanticTokenModifiers
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SemanticTokenModifiers {
    Declaration,
    Definition,
    Readonly,
    Static,
    Deprecated,
    Abstract,
    Async,
    Modification,
    Documentation,
    DefaultLibrary,
}

#[cfg(test)]
mod tests {

    use crate::server::lsp::capabilities::{
        BoolOrEmpty, FullCapability,
        server::{
            CompletionOptions, DiagnosticOptions, DocumentFormattingOptions,
            DocumentOnTypeFormattingOptions, ExecuteCommandOptions, SemanticTokenModifiers,
            SemanticTokenTypes, SemanticTokensOptions, TextDocumentSyncKind,
            WorkDoneProgressOptions,
        },
    };

    use super::ServerCapabilities;

    #[test]
    fn serialize() {
        let server_capabilities = ServerCapabilities {
            text_document_sync: TextDocumentSyncKind::Full,
            hover_provider: true,
            completion_provider: CompletionOptions {
                trigger_characters: vec!["?".to_string()],
            },
            document_formatting_provider: DocumentFormattingOptions {},
            document_on_type_formatting_provider: DocumentOnTypeFormattingOptions {
                first_trigger_character: "\n".to_string(),
                more_trigger_character: None,
            },
            diagnostic_provider: DiagnosticOptions {
                identifier: "my-ls".to_string(),
                inter_file_dependencies: false,
                workspace_diagnostics: false,
            },
            code_action_provider: true,
            execute_command_provider: ExecuteCommandOptions {
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: true,
                },
                commands: vec!["foo".to_string()],
            },
            folding_range_provider: true,
            semantic_tokens_provider: SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: true,
                },
                legend: crate::server::lsp::capabilities::server::SemanticTokensLegend {
                    token_types: vec![SemanticTokenTypes::Function, SemanticTokenTypes::String],
                    token_modifiers: vec![SemanticTokenModifiers::Async],
                },
                range: Some(BoolOrEmpty::Bool(true)),
                full: Some(FullCapability::Bool(true)),
            },
        };

        let serialized = serde_json::to_string(&server_capabilities).unwrap();

        pretty_assertions::assert_eq!(
            serialized,
            r#"{"textDocumentSync":1,"hoverProvider":true,"completionProvider":{"triggerCharacters":["?"]},"documentFormattingProvider":{},"documentOnTypeFormattingProvider":{"firstTriggerCharacter":"\n"},"diagnosticProvider":{"identifier":"my-ls","interFileDependencies":false,"workspaceDiagnostics":false},"codeActionProvider":true,"executeCommandProvider":{"workDoneProgress":true,"commands":["foo"]},"foldingRangeProvider":true,"semanticTokensProvider":{"workDoneProgress":true,"legend":{"tokenTypes":["function","string"],"tokenModifiers":["async"]},"range":true,"full":true}}"#
        );
    }
}
