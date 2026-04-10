use serde::Deserialize;
use serde_repr::Deserialize_repr;

mod completion;
mod semantic_tokens;

pub use completion::CompletionClientCapabilities;
pub use semantic_tokens::SemanticTokensClientCapabilities;

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#clientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    pub workspace: Option<WorkspaceCapabilities>,
    pub text_document: Option<TextDocumentClientCapabilities>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceCapabilities {
    pub apply_edit: Option<bool>,
    pub workspace_edit: Option<WorkspaceEditClientCapabilities>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspaceEditClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEditClientCapabilities {
    pub document_changes: Option<bool>,
}

/// Text document specific client capabilities.
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocumentClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentClientCapabilities {
    /// Capabilities specific to text document synchronization.
    pub synchronization: Option<TextDocumentSyncClientCapabilities>,

    /// Capabilities specific to the `textDocument/completion` request.
    pub completion: Option<CompletionClientCapabilities>,

    /// Capabilities specific to the `textDocument/hover` request.
    pub hover: Option<HoverClientCapabilities>,

    /// Capabilities specific to the `textDocument/signatureHelp` request.
    pub signature_help: Option<SignatureHelpClientCapabilities>,

    /// Capabilities specific to the `textDocument/declaration` request.
    ///
    /// @since 3.14.0
    pub declaration: Option<DeclarationClientCapabilities>,

    /// Capabilities specific to the `textDocument/definition` request.
    pub definition: Option<DefinitionClientCapabilities>,

    /// Capabilities specific to the `textDocument/typeDefinition` request.
    ///
    /// @since 3.6.0
    pub type_definition: Option<TypeDefinitionClientCapabilities>,

    /// Capabilities specific to the `textDocument/implementation` request.
    ///
    /// @since 3.6.0
    pub implementation: Option<ImplementationClientCapabilities>,

    /// Capabilities specific to the `textDocument/references` request.
    pub references: Option<ReferenceClientCapabilities>,

    /// Capabilities specific to the `textDocument/documentHighlight` request.
    pub document_highlight: Option<DocumentHighlightClientCapabilities>,

    /// Capabilities specific to the `textDocument/documentSymbol` request.
    pub document_symbol: Option<DocumentSymbolClientCapabilities>,

    /// Capabilities specific to the `textDocument/codeAction` request.
    pub code_action: Option<CodeActionClientCapabilities>,

    /// Capabilities specific to the `textDocument/codeLens` request.
    pub code_lens: Option<CodeLensClientCapabilities>,

    /// Capabilities specific to the `textDocument/documentLink` request.
    pub document_link: Option<DocumentLinkClientCapabilities>,

    /// Capabilities specific to the `textDocument/documentColor` and the
    /// `textDocument/colorPresentation` request.
    ///
    /// @since 3.6.0
    pub color_provider: Option<DocumentColorClientCapabilities>,

    /// Capabilities specific to the `textDocument/formatting` request.
    pub formatting: Option<DocumentFormattingClientCapabilities>,

    /// Capabilities specific to the `textDocument/rangeFormatting` request.
    pub range_formatting: Option<DocumentRangeFormattingClientCapabilities>,

    /// Capabilities specific to the `textDocument/onTypeFormatting` request.
    pub on_type_formatting: Option<DocumentOnTypeFormattingClientCapabilities>,

    /// Capabilities specific to the `textDocument/rename` request.
    pub rename: Option<RenameClientCapabilities>,

    /// Capabilities specific to the `textDocument/publishDiagnostics`
    /// notification.
    pub publish_diagnostics: Option<PublishDiagnosticsClientCapabilities>,

    /// Capabilities specific to the `textDocument/foldingRange` request.
    ///
    /// @since 3.10.0
    pub folding_range: Option<FoldingRangeClientCapabilities>,

    /// Capabilities specific to the `textDocument/selectionRange` request.
    ///
    /// @since 3.15.0
    pub selection_range: Option<SelectionRangeClientCapabilities>,

    /// Capabilities specific to the `textDocument/linkedEditingRange` request.
    ///
    /// @since 3.16.0
    pub linked_editing_range: Option<LinkedEditingRangeClientCapabilities>,

    /// Capabilities specific to the various call hierarchy requests.
    ///
    /// @since 3.16.0
    pub call_hierarchy: Option<CallHierarchyClientCapabilities>,

    /// Capabilities specific to the various semantic token requests.
    ///
    /// @since 3.16.0
    pub semantic_tokens: Option<SemanticTokensClientCapabilities>,

    /// Capabilities specific to the `textDocument/moniker` request.
    ///
    /// @since 3.16.0
    pub moniker: Option<MonikerClientCapabilities>,

    /// Capabilities specific to the various type hierarchy requests.
    ///
    /// @since 3.17.0
    pub type_hierarchy: Option<TypeHierarchyClientCapabilities>,

    /// Capabilities specific to the `textDocument/inlineValue` request.
    ///
    /// @since 3.17.0
    pub inline_value: Option<InlineValueClientCapabilities>,

    /// Capabilities specific to the `textDocument/inlayHint` request.
    ///
    /// @since 3.17.0
    pub inlay_hint: Option<InlayHintClientCapabilities>,

    /// Capabilities specific to the diagnostic pull model.
    ///
    /// @since 3.17.0
    pub diagnostic: Option<DiagnosticClientCapabilities>,
}

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

/// Describes the content type that a client supports in various result
/// literals like `Hover`, `ParameterInfo` or `CompletionItem`.
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#markupContent
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum MarkupKind {
    /// Plain text is supported as a content format.
    Plaintext,
    /// Markdown is supported as a content format.
    Markdown,
}

/// A value-set wrapper reused by capabilities that advertise supported tags.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueSet<T> {
    pub value_set: Vec<T>,
}

/// A resolve-support object listing lazily resolvable properties.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolveSupport {
    pub properties: Vec<String>,
}

// ---------------------------------------------------------------------------
// Simple capabilities (dynamic_registration only, or + one/two bool fields)
// ---------------------------------------------------------------------------

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocumentSyncClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentSyncClientCapabilities {
    /// Whether text document synchronization supports dynamic registration.
    pub dynamic_registration: Option<bool>,
    /// The client supports sending will save notifications.
    pub will_save: Option<bool>,
    /// The client supports sending a will save request and waits for a
    /// response providing text edits which will be applied to the document
    /// before it is saved.
    pub will_save_wait_until: Option<bool>,
    /// The client supports did save notifications.
    pub did_save: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#hoverClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HoverClientCapabilities {
    /// Whether hover supports dynamic registration.
    pub dynamic_registration: Option<bool>,
    /// Client supports the following content formats if the content property
    /// refers to a `literal of type MarkupContent`. The order describes the
    /// preferred format of the client.
    pub content_format: Option<Vec<MarkupKind>>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#declarationClientCapabilities
///
/// @since 3.14.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeclarationClientCapabilities {
    /// Whether declaration supports dynamic registration. If this is set to
    /// `true` the client supports the new `DeclarationRegistrationOptions`
    /// return value for the corresponding server capability as well.
    pub dynamic_registration: Option<bool>,
    /// The client supports additional metadata in the form of declaration links.
    pub link_support: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#definitionClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionClientCapabilities {
    /// Whether definition supports dynamic registration.
    pub dynamic_registration: Option<bool>,
    /// The client supports additional metadata in the form of definition links.
    ///
    /// @since 3.14.0
    pub link_support: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#typeDefinitionClientCapabilities
///
/// @since 3.6.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TypeDefinitionClientCapabilities {
    /// Whether implementation supports dynamic registration. If this is set to
    /// `true` the client supports the new `TypeDefinitionRegistrationOptions`
    /// return value for the corresponding server capability as well.
    pub dynamic_registration: Option<bool>,
    /// The client supports additional metadata in the form of definition links.
    ///
    /// @since 3.14.0
    pub link_support: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#implementationClientCapabilities
///
/// @since 3.6.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImplementationClientCapabilities {
    /// Whether implementation supports dynamic registration. If this is set to
    /// `true` the client supports the new `ImplementationRegistrationOptions`
    /// return value for the corresponding server capability as well.
    pub dynamic_registration: Option<bool>,
    /// The client supports additional metadata in the form of definition links.
    ///
    /// @since 3.14.0
    pub link_support: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#referenceClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceClientCapabilities {
    /// Whether references supports dynamic registration.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#documentHighlightClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentHighlightClientCapabilities {
    /// Whether document highlight supports dynamic registration.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#codeLensClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CodeLensClientCapabilities {
    /// Whether code lens supports dynamic registration.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#documentLinkClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLinkClientCapabilities {
    /// Whether document link supports dynamic registration.
    pub dynamic_registration: Option<bool>,
    /// Whether the client supports the `tooltip` property on `DocumentLink`.
    ///
    /// @since 3.15.0
    pub tooltip_support: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#documentColorClientCapabilities
///
/// @since 3.6.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentColorClientCapabilities {
    /// Whether document color supports dynamic registration.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#documentFormattingClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFormattingClientCapabilities {
    /// Whether formatting supports dynamic registration.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#documentRangeFormattingClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRangeFormattingClientCapabilities {
    /// Whether range formatting supports dynamic registration.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#documentOnTypeFormattingClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentOnTypeFormattingClientCapabilities {
    /// Whether on type formatting supports dynamic registration.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#selectionRangeClientCapabilities
///
/// @since 3.15.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectionRangeClientCapabilities {
    /// Whether implementation supports dynamic registration for selection
    /// range providers. If this is set to `true` the client supports the new
    /// `SelectionRangeRegistrationOptions` return value for the corresponding
    /// server capability as well.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#linkedEditingRangeClientCapabilities
///
/// @since 3.16.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinkedEditingRangeClientCapabilities {
    /// Whether the implementation supports dynamic registration. If this is
    /// set to `true` the client supports the new
    /// `LinkedEditingRangeRegistrationOptions` return value for the
    /// corresponding server capability as well.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#callHierarchyClientCapabilities
///
/// @since 3.16.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CallHierarchyClientCapabilities {
    /// Whether implementation supports dynamic registration. If this is set to
    /// `true` the client supports the new
    /// `(TextDocumentRegistrationOptions & StaticRegistrationOptions)` return
    /// value for the corresponding server capability as well.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#monikerClientCapabilities
///
/// @since 3.16.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MonikerClientCapabilities {
    /// Whether implementation supports dynamic registration. If this is set to
    /// `true` the client supports the new
    /// `MonikerRegistrationOptions` return value for the corresponding server
    /// capability as well.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#typeHierarchyClientCapabilities
///
/// @since 3.17.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TypeHierarchyClientCapabilities {
    /// Whether implementation supports dynamic registration. If this is set to
    /// `true` the client supports the new
    /// `TypeHierarchyRegistrationOptions` return value for the corresponding
    /// server capability as well.
    pub dynamic_registration: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#inlineValueClientCapabilities
///
/// @since 3.17.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InlineValueClientCapabilities {
    /// Whether implementation supports dynamic registration for inline
    /// value providers.
    pub dynamic_registration: Option<bool>,
}

// ---------------------------------------------------------------------------
// Moderate capabilities (a few nested objects or enums)
// ---------------------------------------------------------------------------

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#signatureHelpClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelpClientCapabilities {
    /// Whether signature help supports dynamic registration.
    pub dynamic_registration: Option<bool>,

    /// The client supports the following `SignatureInformation` specific
    /// properties.
    pub signature_information: Option<SignatureHelpSignatureInformation>,

    /// The client supports to send additional context information for a
    /// `textDocument/signatureHelp` request. A client that opts into
    /// contextSupport will also support the `retriggerCharacters` on
    /// `SignatureHelpOptions`.
    ///
    /// @since 3.15.0
    pub context_support: Option<bool>,
}

/// Nested signature information capabilities.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelpSignatureInformation {
    /// Client supports the following content formats for the documentation
    /// property. The order describes the preferred format of the client.
    pub documentation_format: Option<Vec<MarkupKind>>,

    /// Client capabilities specific to parameter information.
    pub parameter_information: Option<SignatureHelpParameterInformation>,

    /// The client supports the `activeParameter` property on
    /// `SignatureInformation` literal.
    ///
    /// @since 3.16.0
    pub active_parameter_support: Option<bool>,
}

/// Nested parameter information capabilities.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelpParameterInformation {
    /// The client supports processing label offsets instead of a simple label
    /// string.
    ///
    /// @since 3.14.0
    pub label_offset_support: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#documentSymbolClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbolClientCapabilities {
    /// Whether document symbol supports dynamic registration.
    pub dynamic_registration: Option<bool>,

    /// Specific capabilities for the `SymbolKind` in the
    /// `textDocument/documentSymbol` request.
    pub symbol_kind: Option<DocumentSymbolKindCapabilities>,

    /// The client supports hierarchical document symbols.
    pub hierarchical_document_symbol_support: Option<bool>,

    /// The client supports tags on `SymbolInformation`. Tags are supported on
    /// `DocumentSymbol` if `hierarchicalDocumentSymbolSupport` is set to true.
    /// Clients supporting tags have to handle unknown tags gracefully.
    ///
    /// @since 3.16.0
    pub tag_support: Option<ValueSet<SymbolTag>>,

    /// The client supports an additional label presented in the symbol name
    /// of the `DocumentSymbol` to differentiate symbols with the same name.
    ///
    /// @since 3.16.0
    pub label_support: Option<bool>,
}

/// The `symbolKind` capabilities for document symbols.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbolKindCapabilities {
    /// The symbol kind values the client supports. When this property exists
    /// the client also guarantees that it will handle values outside its set
    /// gracefully and falls back to a default value when unknown.
    ///
    /// If this property is not present the client only supports the symbol
    /// kinds from `File` to `Array` as defined in the initial version of the
    /// protocol.
    pub value_set: Option<Vec<SymbolKind>>,
}

/// A symbol kind.
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#symbolKind
#[derive(Debug, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum SymbolKind {
    File = 1,
    Module = 2,
    Namespace = 3,
    Package = 4,
    Class = 5,
    Method = 6,
    Property = 7,
    Field = 8,
    Constructor = 9,
    Enum = 10,
    Interface = 11,
    Function = 12,
    Variable = 13,
    Constant = 14,
    String = 15,
    Number = 16,
    Boolean = 17,
    Array = 18,
    Object = 19,
    Key = 20,
    Null = 21,
    EnumMember = 22,
    Struct = 23,
    Event = 24,
    Operator = 25,
    TypeParameter = 26,
}

/// Symbol tags are extra annotations that tweak the rendering of a symbol.
///
/// @since 3.16.0
#[derive(Debug, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum SymbolTag {
    /// Render a symbol as obsolete, usually using a strike-out.
    Deprecated = 1,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#codeActionClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionClientCapabilities {
    /// Whether code action supports dynamic registration.
    pub dynamic_registration: Option<bool>,

    /// The client supports code action literals as a valid response of the
    /// `textDocument/codeAction` request.
    ///
    /// @since 3.8.0
    pub code_action_literal_support: Option<CodeActionLiteralSupport>,

    /// Whether code action supports the `isPreferred` property.
    ///
    /// @since 3.15.0
    pub is_preferred_support: Option<bool>,

    /// Whether code action supports the `disabled` property.
    ///
    /// @since 3.16.0
    pub disabled_support: Option<bool>,

    /// Whether code action supports the `data` property which is preserved
    /// between a `textDocument/codeAction` and a `codeAction/resolve` request.
    ///
    /// @since 3.16.0
    pub data_support: Option<bool>,

    /// Whether the client supports resolving additional code action properties
    /// via a separate `codeAction/resolve` request.
    ///
    /// @since 3.16.0
    pub resolve_support: Option<ResolveSupport>,

    /// Whether the client honors the change annotations in text edits and
    /// resource operations returned via the `CodeAction#edit` property by for
    /// example presenting the workspace edit in the user interface and asking
    /// for confirmation.
    ///
    /// @since 3.16.0
    pub honors_change_annotations: Option<bool>,
}

/// Code action literal support.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionLiteralSupport {
    /// The code action kind is supported with the following value set.
    pub code_action_kind: CodeActionKindCapabilities,
}

/// The supported code action kind values.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionKindCapabilities {
    /// The code action kind values the client supports. When this property
    /// exists the client also guarantees that it will handle values outside its
    /// set gracefully and falls back to a default value when unknown.
    pub value_set: Vec<String>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#renameClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RenameClientCapabilities {
    /// Whether rename supports dynamic registration.
    pub dynamic_registration: Option<bool>,

    /// Client supports testing for validity of rename operations before
    /// execution.
    ///
    /// @since version 3.12.0
    pub prepare_support: Option<bool>,

    /// Client supports the default behavior result
    /// (`{ defaultBehavior: boolean }`).
    ///
    /// The value indicates the default behavior used by the client.
    ///
    /// @since version 3.16.0
    pub prepare_support_default_behavior: Option<PrepareSupportDefaultBehavior>,

    /// Whether the client honors the change annotations in text edits and
    /// resource operations returned via the rename request's workspace edit by
    /// for example presenting the workspace edit in the user interface and
    /// asking for confirmation.
    ///
    /// @since 3.16.0
    pub honors_change_annotations: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#prepareSupportDefaultBehavior
#[derive(Debug, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum PrepareSupportDefaultBehavior {
    /// The client's default behavior is to select the identifier according to
    /// the language's syntax rule.
    Identifier = 1,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#publishDiagnosticsClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublishDiagnosticsClientCapabilities {
    /// Whether the clients accepts diagnostics with related information.
    pub related_information: Option<bool>,

    /// Client supports the tag property to provide meta data about a
    /// diagnostic. Clients supporting tags have to handle unknown tags
    /// gracefully.
    ///
    /// @since 3.15.0
    pub tag_support: Option<ValueSet<DiagnosticTag>>,

    /// Whether the client interprets the version property of the
    /// `textDocument/publishDiagnostics` notification's parameter.
    ///
    /// @since 3.15.0
    pub version_support: Option<bool>,

    /// Client supports a codeDescription property.
    ///
    /// @since 3.16.0
    pub code_description_support: Option<bool>,

    /// Whether code action supports the `data` property which is preserved
    /// between a `textDocument/publishDiagnostics` and
    /// `textDocument/codeAction` request.
    ///
    /// @since 3.16.0
    pub data_support: Option<bool>,
}

/// The diagnostic tags.
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnosticTag
#[derive(Debug, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum DiagnosticTag {
    /// Unused or unnecessary code.
    /// Clients are allowed to render diagnostics with this tag faded out
    /// instead of having an error squiggle.
    Unnecessary = 1,
    /// Deprecated or obsolete code.
    /// Clients are allowed to rendered diagnostics with this tag strike through.
    Deprecated = 2,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#foldingRangeClientCapabilities
///
/// @since 3.10.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FoldingRangeClientCapabilities {
    /// Whether implementation supports dynamic registration for folding range
    /// providers. If this is set to `true` the client supports the new
    /// `FoldingRangeRegistrationOptions` return value for the corresponding
    /// server capability as well.
    pub dynamic_registration: Option<bool>,

    /// The maximum number of folding ranges that the client prefers to receive
    /// per document. The value serves as a hint, servers are free to follow
    /// the limit.
    pub range_limit: Option<u32>,

    /// If set, the client signals that it only supports folding complete lines.
    /// If set, client will ignore specified `startCharacter` and `endCharacter`
    /// properties in a FoldingRange.
    pub line_folding_only: Option<bool>,

    /// Specific options for the folding range kind.
    ///
    /// @since 3.17.0
    pub folding_range_kind: Option<FoldingRangeKindCapabilities>,

    /// Specific options for the folding range.
    ///
    /// @since 3.17.0
    pub folding_range: Option<FoldingRangeCapabilities>,
}

/// The folding range kind capabilities.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FoldingRangeKindCapabilities {
    /// The folding range kind values the client supports. When this property
    /// exists the client also guarantees that it will handle values outside its
    /// set gracefully and falls back to a default value when unknown.
    pub value_set: Option<Vec<FoldingRangeKind>>,
}

/// Known folding range kinds.
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#foldingRangeKind
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum FoldingRangeKind {
    /// Folding range for a comment.
    Comment,
    /// Folding range for imports or includes.
    Imports,
    /// Folding range for a region (e.g. `#region`).
    Region,
}

/// Specific folding range capabilities.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FoldingRangeCapabilities {
    /// If set, the client signals that it supports setting `collapsedText` on
    /// folding ranges to display custom labels instead of a default text.
    ///
    /// @since 3.17.0
    pub collapsed_text: Option<bool>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#inlayHintClientCapabilities
///
/// @since 3.17.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InlayHintClientCapabilities {
    /// Whether inlay hints support dynamic registration.
    pub dynamic_registration: Option<bool>,

    /// Indicates which properties a client can resolve lazily on an inlay hint.
    pub resolve_support: Option<ResolveSupport>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnosticClientCapabilities
///
/// @since 3.17.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticClientCapabilities {
    /// Whether implementation supports dynamic registration. If this is set to
    /// `true` the client supports the new
    /// `(TextDocumentRegistrationOptions & StaticRegistrationOptions)` return
    /// value for the corresponding server capability as well.
    pub dynamic_registration: Option<bool>,

    /// Whether the clients supports related documents for document diagnostic
    /// pulls.
    pub related_document_support: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_text_document_client_capabilities() {
        let json = r#"{
            "synchronization": {
                "dynamicRegistration": true,
                "willSave": true,
                "willSaveWaitUntil": true,
                "didSave": true
            },
            "completion": {
                "dynamicRegistration": true,
                "completionItem": {
                    "snippetSupport": true,
                    "commitCharactersSupport": true,
                    "documentationFormat": ["markdown", "plaintext"],
                    "deprecatedSupport": true,
                    "preselectSupport": true,
                    "insertReplaceSupport": true,
                    "labelDetailsSupport": true
                },
                "completionItemKind": {
                    "valueSet": [1, 2, 3, 4, 5, 6]
                },
                "contextSupport": true
            },
            "hover": {
                "dynamicRegistration": true,
                "contentFormat": ["markdown", "plaintext"]
            },
            "signatureHelp": {
                "dynamicRegistration": true,
                "signatureInformation": {
                    "documentationFormat": ["markdown"],
                    "parameterInformation": {
                        "labelOffsetSupport": true
                    },
                    "activeParameterSupport": true
                },
                "contextSupport": true
            },
            "definition": {
                "dynamicRegistration": true,
                "linkSupport": true
            },
            "documentSymbol": {
                "dynamicRegistration": true,
                "symbolKind": {
                    "valueSet": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]
                },
                "hierarchicalDocumentSymbolSupport": true,
                "labelSupport": true
            },
            "codeAction": {
                "dynamicRegistration": true,
                "codeActionLiteralSupport": {
                    "codeActionKind": {
                        "valueSet": ["quickfix", "refactor", "source"]
                    }
                },
                "isPreferredSupport": true,
                "dataSupport": true,
                "resolveSupport": {
                    "properties": ["edit"]
                }
            },
            "formatting": {
                "dynamicRegistration": true
            },
            "publishDiagnostics": {
                "relatedInformation": true,
                "tagSupport": {
                    "valueSet": [1, 2]
                },
                "versionSupport": true,
                "codeDescriptionSupport": true,
                "dataSupport": true
            },
            "foldingRange": {
                "dynamicRegistration": true,
                "rangeLimit": 5000,
                "lineFoldingOnly": true,
                "foldingRangeKind": {
                    "valueSet": ["comment", "imports", "region"]
                },
                "foldingRange": {
                    "collapsedText": true
                }
            },
            "rename": {
                "dynamicRegistration": true,
                "prepareSupport": true,
                "prepareSupportDefaultBehavior": 1,
                "honorsChangeAnnotations": true
            },
            "diagnostic": {
                "dynamicRegistration": true,
                "relatedDocumentSupport": true
            }
        }"#;

        let caps: TextDocumentClientCapabilities = serde_json::from_str(json).unwrap();

        let sync = caps.synchronization.unwrap();
        assert_eq!(sync.dynamic_registration, Some(true));
        assert_eq!(sync.will_save, Some(true));
        assert_eq!(sync.will_save_wait_until, Some(true));
        assert_eq!(sync.did_save, Some(true));

        let completion = caps.completion.unwrap();
        assert_eq!(completion.dynamic_registration, Some(true));
        assert_eq!(completion.context_support, Some(true));
        let item = completion.completion_item.unwrap();
        assert_eq!(item.snippet_support, Some(true));
        assert_eq!(
            item.documentation_format,
            Some(vec![MarkupKind::Markdown, MarkupKind::Plaintext])
        );
        assert_eq!(item.label_details_support, Some(true));
        let item_kind = completion.completion_item_kind.unwrap();
        assert_eq!(item_kind.value_set.unwrap().len(), 6);

        let hover = caps.hover.unwrap();
        assert_eq!(hover.dynamic_registration, Some(true));
        assert_eq!(
            hover.content_format,
            Some(vec![MarkupKind::Markdown, MarkupKind::Plaintext])
        );

        let sig = caps.signature_help.unwrap();
        assert_eq!(sig.context_support, Some(true));
        let sig_info = sig.signature_information.unwrap();
        assert_eq!(sig_info.active_parameter_support, Some(true));
        assert_eq!(
            sig_info.parameter_information.unwrap().label_offset_support,
            Some(true)
        );

        let def = caps.definition.unwrap();
        assert_eq!(def.link_support, Some(true));

        let doc_sym = caps.document_symbol.unwrap();
        assert_eq!(doc_sym.hierarchical_document_symbol_support, Some(true));
        assert_eq!(doc_sym.symbol_kind.unwrap().value_set.unwrap().len(), 13);

        let code_action = caps.code_action.unwrap();
        assert_eq!(code_action.is_preferred_support, Some(true));
        assert_eq!(code_action.data_support, Some(true));
        assert_eq!(
            code_action.resolve_support.unwrap().properties,
            vec!["edit".to_string()]
        );
        assert_eq!(
            code_action
                .code_action_literal_support
                .unwrap()
                .code_action_kind
                .value_set,
            vec![
                "quickfix".to_string(),
                "refactor".to_string(),
                "source".to_string()
            ]
        );

        let formatting = caps.formatting.unwrap();
        assert_eq!(formatting.dynamic_registration, Some(true));

        let pub_diag = caps.publish_diagnostics.unwrap();
        assert_eq!(pub_diag.related_information, Some(true));
        assert_eq!(pub_diag.version_support, Some(true));
        assert_eq!(pub_diag.code_description_support, Some(true));
        assert_eq!(
            pub_diag.tag_support.unwrap().value_set,
            vec![DiagnosticTag::Unnecessary, DiagnosticTag::Deprecated]
        );

        let folding = caps.folding_range.unwrap();
        assert_eq!(folding.range_limit, Some(5000));
        assert_eq!(folding.line_folding_only, Some(true));
        assert_eq!(
            folding.folding_range_kind.unwrap().value_set,
            Some(vec![
                FoldingRangeKind::Comment,
                FoldingRangeKind::Imports,
                FoldingRangeKind::Region,
            ])
        );
        assert_eq!(folding.folding_range.unwrap().collapsed_text, Some(true));

        let rename = caps.rename.unwrap();
        assert_eq!(rename.prepare_support, Some(true));
        assert_eq!(
            rename.prepare_support_default_behavior,
            Some(PrepareSupportDefaultBehavior::Identifier)
        );
        assert_eq!(rename.honors_change_annotations, Some(true));

        let diagnostic = caps.diagnostic.unwrap();
        assert_eq!(diagnostic.related_document_support, Some(true));

        // Fields not present in the JSON should be None.
        assert!(caps.declaration.is_none());
        assert!(caps.type_definition.is_none());
        assert!(caps.implementation.is_none());
        assert!(caps.references.is_none());
        assert!(caps.document_highlight.is_none());
        assert!(caps.code_lens.is_none());
        assert!(caps.document_link.is_none());
        assert!(caps.color_provider.is_none());
        assert!(caps.range_formatting.is_none());
        assert!(caps.on_type_formatting.is_none());
        assert!(caps.selection_range.is_none());
        assert!(caps.linked_editing_range.is_none());
        assert!(caps.call_hierarchy.is_none());
        assert!(caps.semantic_tokens.is_none());
        assert!(caps.moniker.is_none());
        assert!(caps.type_hierarchy.is_none());
        assert!(caps.inline_value.is_none());
        assert!(caps.inlay_hint.is_none());
    }
}
