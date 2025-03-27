use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::server::lsp::{
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::TextEdit,
};

use super::utils::TextDocumentPositionParams;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
        return &self.params.context;
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CompletionParams {
    #[serde(flatten)]
    base: TextDocumentPositionParams,
    pub context: CompletionContext,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompletionContext {
    pub trigger_kind: CompletionTriggerKind,
    pub trigger_character: Option<String>,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum CompletionTriggerKind {
    Invoked = 1,
    TriggerCharacter = 2,
    TriggerForIncompleteCompletions = 3,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CompletionResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: CompletionResult,
}

impl CompletionResponse {
    pub fn new(id: &RequestId, items: Vec<CompletionItem>) -> Self {
        CompletionResponse {
            base: ResponseMessageBase::success(id),
            result: CompletionResult { items },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CompletionResult {
    items: Vec<CompletionItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub sort_text: Option<String>,
    pub insert_text: Option<String>,
    pub text_edit: Option<TextEdit>,
    pub insert_text_format: InsertTextFormat,
    pub additional_text_edits: Option<Vec<TextEdit>>,
}

impl CompletionItem {
    pub fn new(
        label: &str,
        detail: Option<String>,
        sort_text: Option<String>,
        insert_text: &str,
        kind: CompletionItemKind,
        insert_text_format: InsertTextFormat,
        additional_text_edits: Option<Vec<TextEdit>>,
    ) -> Self {
        Self {
            label: label.to_string(),
            kind,
            detail,
            sort_text,
            insert_text: Some(insert_text.to_string()),
            text_edit: None,
            insert_text_format,
            additional_text_edits,
        }
    }
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq)]
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
        CompletionContext, CompletionItem, CompletionItemKind, CompletionParams,
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
        let cmp = CompletionItem::new(
            "SELECT",
            "Select query",
            "SELECT ${1:*} WHERE {\n  $0\n}",
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
            None,
        );
        let completion_response = CompletionResponse::new(&RequestId::Integer(1337), vec![cmp]);
        let expected_message = r#"{"jsonrpc":"2.0","id":1337,"result":{"items":[{"label":"SELECT","kind":15,"detail":"Select query","insertText":"SELECT ${1:*} WHERE {\n  $0\n}","insertTextFormat":2,"additionalTextEdits":null}]}}"#;
        let actual_message = serde_json::to_string(&completion_response).unwrap();
        assert_eq!(actual_message, expected_message);
    }
}
