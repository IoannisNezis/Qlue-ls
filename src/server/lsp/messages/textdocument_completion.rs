use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::server::lsp::{
    LspMessage,
    base_types::LSPAny,
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::{Range, TextEdit},
};

use super::{command::Command, utils::TextDocumentPositionParams};

#[derive(Debug, Deserialize, PartialEq)]
pub struct CompletionRequest {
    #[serde(flatten)]
    base: RequestMessageBase,
    pub params: CompletionParams,
}

impl LspMessage for CompletionRequest {}

impl CompletionRequest {
    pub(crate) fn get_text_position(&self) -> &TextDocumentPositionParams {
        &self.params.base
    }

    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }

    pub(crate) fn get_completion_context(&self) -> &CompletionContext {
        &self.params.context
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct CompletionParams {
    #[serde(flatten)]
    base: TextDocumentPositionParams,
    pub context: CompletionContext,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompletionContext {
    pub trigger_kind: CompletionTriggerKind,
    pub trigger_character: Option<String>,
}

#[derive(Debug, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum CompletionTriggerKind {
    Invoked = 1,
    TriggerCharacter = 2,
    TriggerForIncompleteCompletions = 3,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct CompletionResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: CompletionList,
}

impl LspMessage for CompletionResponse {}

impl CompletionResponse {
    pub fn new(id: &RequestId, completion_list: CompletionList) -> Self {
        CompletionResponse {
            base: ResponseMessageBase::success(id),
            result: completion_list,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompletionList {
    pub is_incomplete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_defaults: Option<ItemDefaults>,
    pub items: Vec<CompletionItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemDefaults {
    /// A default commit character set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_characters: Option<Vec<String>>,

    /// A default edit range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_range: Option<Range>,

    /// A default insert text format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text_format: Option<InsertTextFormat>,

    /// A default insert text mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text_mode: Option<InsertTextMode>,

    /// A default data value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<LSPAny>,
}

/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#completionItem
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItem {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_details: Option<CompletionItemLabelDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<CompletionItemKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_edit: Option<TextEdit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text_format: Option<InsertTextFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_text_edits: Option<Vec<TextEdit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Command>,
}

impl CompletionItem {
    pub fn new(
        label: &str,
        detail: Option<String>,
        sort_text: Option<String>,
        insert_text: &str,
        kind: CompletionItemKind,
        additional_text_edits: Option<Vec<TextEdit>>,
    ) -> Self {
        Self {
            label: label.to_string(),
            label_details: None,
            kind: Some(kind),
            detail,
            documentation: None,
            sort_text,
            filter_text: None,
            insert_text: Some(insert_text.to_string()),
            text_edit: None,
            insert_text_format: None,
            additional_text_edits,
            command: None,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct CompletionItemLabelDetails {
    pub detail: String,
}

#[derive(Debug, Serialize_repr, PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
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

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum InsertTextFormat {
    PlainText = 1,
    Snippet = 2,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum InsertTextMode {
    AsIs = 1,
    AdjustIndentation = 2,
}

#[derive(Debug, Default)]
pub struct CompletionItemBuilder {
    pub label: Option<String>,
    pub label_details: Option<CompletionItemLabelDetails>,
    pub kind: Option<CompletionItemKind>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub sort_text: Option<String>,
    pub filter_text: Option<String>,
    pub insert_text: Option<String>,
    pub text_edit: Option<TextEdit>,
    pub insert_text_format: Option<InsertTextFormat>,
    pub additional_text_edits: Option<Vec<TextEdit>>,
    pub command: Option<Command>,
}

impl CompletionItemBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the label of the completion.
    /// The label property is also by default the text that
    /// is inserted when selecting this completion.
    /// If label details are provided the label itself should
    /// be an unqualified name of the completion item.
    pub fn label(mut self, label: &str) -> CompletionItemBuilder {
        self.label = Some(label.to_string());
        self
    }

    /// Set the label_detail of the completion.
    /// This is the text unprominently displayed to the user to describe the completion.
    pub fn label_details(mut self, detail: &str) -> CompletionItemBuilder {
        self.label_details = Some(CompletionItemLabelDetails {
            detail: detail.to_string(),
        });
        self
    }

    /// Set the kind of the completion.
    /// The kind of this completion item. Based of the kind
    /// an icon is chosen by the editor. The standardized set
    /// of available values is defined in `CompletionItemKind`.
    ///
    pub fn kind(mut self, kind: CompletionItemKind) -> CompletionItemBuilder {
        self.kind = Some(kind);
        self
    }

    /// Set the kind of the completion.
    ///
    /// A human-readable string with additional information
    /// about this item, like type or symbol information.
    pub fn detail(mut self, detail: &str) -> CompletionItemBuilder {
        self.detail = Some(detail.to_string());
        self
    }

    /// Set the documentation text of the completion.
    ///
    /// A human-readable string that represents a doc-comment.
    pub fn documentation(mut self, documentation: &str) -> CompletionItemBuilder {
        self.documentation = Some(documentation.to_string());
        self
    }

    /// Set the sort text of the completion.
    ///
    /// A string that should be used when comparing this item
    /// with other items. When omitted the label is used
    /// as the sort text for this item.
    pub fn sort_text(mut self, sort_text: &str) -> CompletionItemBuilder {
        self.sort_text = Some(sort_text.to_string());
        self
    }

    /// Set the filter text of the completion.
    ///
    /// A string that should be used when filtering a set of
    /// completion items. When omitted the label is used as the
    /// filter text for this item.
    pub fn filter_text(mut self, filter_text: &str) -> CompletionItemBuilder {
        self.filter_text = Some(filter_text.to_string());
        self
    }

    /// Set the insert text of the completion.
    ///
    /// A string that should be inserted into a document when selecting
    /// this completion. When omitted the label is used as the insert text
    /// for this item.
    ///
    /// The `insertText` is subject to interpretation by the client side.
    /// Some tools might not take the string literally. For example
    /// VS Code when code complete is requested in this example
    /// `con<cursor position>` and a completion item with an `insertText` of
    /// `console` is provided it will only insert `sole`. Therefore it is
    /// recommended to use `textEdit` instead since it avoids additional client
    /// side interpretation.
    pub fn insert_text(mut self, insert_text: &str) -> CompletionItemBuilder {
        self.insert_text = Some(insert_text.to_string());
        self
    }

    /// Set the text edit of the completion.
    ///
    /// An edit which is applied to a document when selecting this completion.
    /// When an edit is provided the value of `insertText` is ignored.
    ///
    /// *Note:* The range of the edit must be a single line range and it must
    /// contain the position at which completion has been requested.
    ///
    /// Most editors support two different operations when accepting a completion
    /// item. One is to insert a completion text and the other is to replace an
    /// existing text with a completion text. Since this can usually not be
    /// predetermined by a server it can report both ranges. Clients need to
    /// signal support for `InsertReplaceEdit`s via the
    /// `textDocument.completion.completionItem.insertReplaceSupport` client
    /// capability property.
    ///
    /// *Note 1:* The text edit's range as well as both ranges from an insert
    /// replace edit must be a [single line] and they must contain the position
    /// at which completion has been requested.
    /// *Note 2:* If an `InsertReplaceEdit` is returned the edit's insert range
    /// must be a prefix of the edit's replace range, that means it must be
    /// contained and starting at the same position.
    ///
    /// @since 3.16.0 additional type `InsertReplaceEdit`
    pub fn text_edit(mut self, text_edit: TextEdit) -> CompletionItemBuilder {
        self.text_edit = Some(text_edit);
        self
    }

    /// Set the insert text format of the completion.
    ///
    /// How whitespace and indentation is handled during completion
    /// item insertion. If not provided the client's default value depends on
    /// the `textDocument.completion.insertTextMode` client capability.
    ///
    /// @since 3.16.0
    /// @since 3.17.0 - support for `textDocument.completion.insertTextMode`
    ///
    pub fn insert_text_format(
        mut self,
        insert_text_format: InsertTextFormat,
    ) -> CompletionItemBuilder {
        self.insert_text_format = Some(insert_text_format);
        self
    }

    /// Set additional text edits of the completion.
    ///
    /// An optional array of additional text edits that are applied when
    /// selecting this completion. Edits must not overlap (including the same
    /// insert position) with the main edit nor with themselves.
    ///
    /// Additional text edits should be used to change text unrelated to the
    /// current cursor position (for example adding an import statement at the
    /// top of the file if the completion item will insert an unqualified type).
    pub fn additional_text_edits(mut self, text_edits: Vec<TextEdit>) -> CompletionItemBuilder {
        self.additional_text_edits = Some(text_edits);
        self
    }

    /// Set the command of the completion.
    ///
    /// An optional command that is executed *after* inserting this completion.
    /// *Note* that additional modifications to the current document should be
    /// described with the additionalTextEdits-property.
    pub fn command(mut self, command: Command) -> CompletionItemBuilder {
        self.command = Some(command);
        self
    }

    /// Build the completion item.
    /// **Panics** if the label has not been set.
    pub fn build(self) -> CompletionItem {
        CompletionItem {
            label: self.label.expect("Label should have been set."),
            label_details: self.label_details,
            kind: self.kind,
            detail: self.detail,
            documentation: self.documentation,
            sort_text: self.sort_text,
            filter_text: self.filter_text,
            insert_text: self.insert_text,
            text_edit: self.text_edit,
            insert_text_format: self.insert_text_format,
            additional_text_edits: self.additional_text_edits,
            command: self.command,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::server::lsp::{
        CompletionContext, CompletionItem, CompletionItemKind, CompletionList, CompletionParams,
        CompletionTriggerKind, InsertTextFormat,
        messages::utils::TextDocumentPositionParams,
        rpc::{Message, RequestId, RequestMessageBase},
        textdocument::{Position, TextDocumentIdentifier},
    };

    use super::{CompletionRequest, CompletionResponse};

    #[test]
    fn deserialize() {
        let message = br#"{"id":4,"params":{"position":{"line":0,"character":0},"context":{"triggerKind":1},"textDocument":{"uri":"file:///dings"}},"jsonrpc":"2.0","method":"textDocument/completion"}"#;
        let completion_request: CompletionRequest = serde_json::from_slice(message).unwrap();

        assert_eq!(
            completion_request,
            CompletionRequest {
                base: RequestMessageBase {
                    base: Message {
                        jsonrpc: "2.0".to_string()
                    },
                    method: "textDocument/completion".to_string(),
                    id: RequestId::Integer(4)
                },
                params: CompletionParams {
                    base: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier {
                            uri: "file:///dings".to_string()
                        },
                        position: Position::new(0, 0)
                    },
                    context: CompletionContext {
                        trigger_kind: CompletionTriggerKind::Invoked,
                        trigger_character: None
                    }
                }
            }
        )
    }

    #[test]
    fn serialize() {
        let cmp = CompletionItem {
            command: None,
            label: "SELECT".to_string(),
            label_details: None,
            detail: Some("Select query".to_string()),
            documentation: None,
            sort_text: None,
            filter_text: None,
            insert_text: Some("SELECT ${1:*} WHERE {\n  $0\n}".to_string()),
            text_edit: None,
            kind: Some(CompletionItemKind::Snippet),
            insert_text_format: Some(InsertTextFormat::Snippet),
            additional_text_edits: None,
        };
        let completion_list = CompletionList {
            is_incomplete: true,
            item_defaults: None,
            items: vec![cmp],
        };
        let completion_response =
            CompletionResponse::new(&RequestId::Integer(1337), completion_list);
        let expected_message = r#"{"jsonrpc":"2.0","id":1337,"result":{"isIncomplete":true,"items":[{"label":"SELECT","kind":15,"detail":"Select query","insertText":"SELECT ${1:*} WHERE {\n  $0\n}","insertTextFormat":2}]}}"#;
        let actual_message = serde_json::to_string(&completion_response).unwrap();
        assert_eq!(actual_message, expected_message);
    }
}
