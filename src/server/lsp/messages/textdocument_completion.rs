use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::server::lsp::{
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
    result: Option<CompletionList>,
}

impl CompletionResponse {
    pub fn new(id: &RequestId, completion_list: Option<CompletionList>) -> Self {
        CompletionResponse {
            base: ResponseMessageBase::success(id),
            result: completion_list,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
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

    /// A default data value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<LSPAny>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItem {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_details: Option<CompletionItemLabelDetails>,
    pub kind: CompletionItemKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_text: Option<String>,
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
            kind,
            detail,
            sort_text,
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

#[cfg(test)]
mod tests {
    use crate::server::lsp::{
        messages::utils::TextDocumentPositionParams,
        rpc::{Message, RequestId, RequestMessageBase},
        textdocument::{Position, TextDocumentIdentifier},
        CompletionContext, CompletionItem, CompletionItemKind, CompletionList, CompletionParams,
        CompletionTriggerKind, InsertTextFormat,
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
            sort_text: None,
            insert_text: Some("SELECT ${1:*} WHERE {\n  $0\n}".to_string()),
            text_edit: None,
            kind: CompletionItemKind::Snippet,
            insert_text_format: Some(InsertTextFormat::Snippet),
            additional_text_edits: None,
        };
        let completion_list = CompletionList {
            is_incomplete: true,
            item_defaults: None,
            items: vec![cmp],
        };
        let completion_response =
            CompletionResponse::new(&RequestId::Integer(1337), Some(completion_list));
        let expected_message = r#"{"jsonrpc":"2.0","id":1337,"result":{"isIncomplete":true,"items":[{"label":"SELECT","kind":15,"detail":"Select query","insertText":"SELECT ${1:*} WHERE {\n  $0\n}","insertTextFormat":2}]}}"#;
        let actual_message = serde_json::to_string(&completion_response).unwrap();
        assert_eq!(actual_message, expected_message);
    }
}
