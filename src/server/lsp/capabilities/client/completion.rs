use serde::Deserialize;
use serde_repr::Deserialize_repr;

use super::MarkupKind;

/// Capabilities specific to the `textDocument/completion` request.
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#completionClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletionClientCapabilities {
    /// Whether completion supports dynamic registration.
    pub dynamic_registration: Option<bool>,

    /// The client supports the following `CompletionItem` specific capabilities.
    pub completion_item: Option<CompletionClientCapabilitiesCompletionItem>,

    /// Specific capabilities for the `CompletionItemKind` in the
    /// `textDocument/completion` request.
    pub completion_item_kind: Option<CompletionClientCapabilitiesCompletionItemKind>,

    /// The client supports to send additional context information for a
    /// `textDocument/completion` request.
    pub context_support: Option<bool>,

    /// The client's default when the completion item doesn't provide an
    /// `insertTextMode` property.
    ///
    /// @since 3.17.0
    pub insert_text_mode: Option<InsertTextMode>,

    /// The client supports the following `CompletionList` specific capabilities.
    ///
    /// @since 3.17.0
    pub completion_list: Option<CompletionClientCapabilitiesCompletionList>,
}

/// Additional properties that describe a completion item.
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#completionClientCapabilities
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletionClientCapabilitiesCompletionItem {
    /// Client supports snippets as insert text.
    ///
    /// A snippet can define tab stops and placeholders with `$1`, `$2`
    /// and `${3:foo}`. `$0` defines the final tab stop, it defaults to
    /// the end of the snippet. Placeholders with equal identifiers are
    /// linked, that is typing in one will update others too.
    pub snippet_support: Option<bool>,

    /// Client supports commit characters on a completion item.
    pub commit_characters_support: Option<bool>,

    /// Client supports the following content formats for the documentation
    /// property. The order describes the preferred format of the client.
    pub documentation_format: Option<Vec<MarkupKind>>,

    /// Client supports the deprecated property on a completion item.
    pub deprecated_support: Option<bool>,

    /// Client supports the preselect property on a completion item.
    pub preselect_support: Option<bool>,

    /// Client supports the tag property on a completion item. Clients
    /// supporting tags have to handle unknown tags gracefully. Clients
    /// especially need to preserve unknown tags when sending a completion
    /// item back to the server in a `completionItem/resolve` request.
    ///
    /// @since 3.15.0
    pub tag_support: Option<CompletionItemTagSupport>,

    /// Client supports insert replace edit to control different behavior if
    /// a completion item is inserted in the text or should replace text.
    ///
    /// @since 3.16.0
    pub insert_replace_support: Option<bool>,

    /// Indicates which properties a client can resolve lazily on a
    /// completion item. Before version 3.16.0 only the predefined properties
    /// `documentation` and `detail` could be resolved lazily.
    ///
    /// @since 3.16.0
    pub resolve_support: Option<CompletionItemResolveSupport>,

    /// The client supports the `insertTextMode` property on a completion
    /// item to override the whitespace handling mode as defined by the client
    /// (see `insertTextMode`).
    ///
    /// @since 3.16.0
    pub insert_text_mode_support: Option<CompletionItemInsertTextModeSupport>,

    /// The client has support for completion item label details (see also
    /// `CompletionItemLabelDetails`).
    ///
    /// @since 3.17.0
    pub label_details_support: Option<bool>,
}

/// Tag support for completion items.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItemTagSupport {
    /// The tags supported by the client.
    pub value_set: Vec<CompletionItemTag>,
}

/// Resolve support for completion items.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItemResolveSupport {
    /// The properties that a client can resolve lazily.
    pub properties: Vec<String>,
}

/// Insert text mode support for completion items.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItemInsertTextModeSupport {
    pub value_set: Vec<InsertTextMode>,
}

/// Specific capabilities for the `CompletionItemKind`.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletionClientCapabilitiesCompletionItemKind {
    /// The completion item kind values the client supports. When this
    /// property exists the client also guarantees that it will
    /// handle values outside its set gracefully and falls back
    /// to a default value when unknown.
    ///
    /// If this property is not present the client only supports
    /// the completion items kinds from `Text` to `Reference` as defined in
    /// the initial version of the protocol.
    pub value_set: Option<Vec<CompletionItemKind>>,
}

/// Client supports the following `CompletionList` specific capabilities.
///
/// @since 3.17.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletionClientCapabilitiesCompletionList {
    /// The client supports the following itemDefaults on a completion list.
    ///
    /// The value lists the supported property names of the
    /// `CompletionList.itemDefaults` object. If omitted no properties are
    /// supported.
    ///
    /// @since 3.17.0
    pub item_defaults: Option<Vec<String>>,
}

/// Completion item tags are extra annotations that tweak the rendering of a
/// completion item.
///
/// @since 3.15.0
#[derive(Debug, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum CompletionItemTag {
    /// Render a completion as obsolete, usually using a strike-out.
    Deprecated = 1,
}

/// How whitespace and indentation is handled during completion item insertion.
///
/// @since 3.16.0
#[derive(Debug, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum InsertTextMode {
    /// The insertion or replace strings are taken as-is. If the value is multi
    /// line the lines below the cursor will be inserted using the indentation
    /// defined in the string value. The client will not apply any kind of
    /// adjustments to the string.
    AsIs = 1,

    /// The editor adjusts leading whitespace of new lines so that they match
    /// the indentation up to the cursor of the line for which the item is
    /// accepted.
    ///
    /// Consider a line like this: `<2tabs><cursor><3tabs>foo`. Accepting a
    /// multi line completion item is indented using 2 tabs and all following
    /// lines inserted will be indented using 2 tabs as well.
    AdjustIndentation = 2,
}

/// The kind of a completion entry.
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#completionItemKind
#[derive(Debug, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}
