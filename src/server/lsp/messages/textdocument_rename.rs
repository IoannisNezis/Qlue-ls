use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    LspMessage,
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::Position,
};

use super::{WorkspaceEdit, utils::TextDocumentPositionParams};

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_rename
#[derive(Debug, Deserialize, PartialEq)]
pub struct RenameRequest {
    #[serde(flatten)]
    base: RequestMessageBase,
    params: RenameParams,
}

impl LspMessage for RenameRequest {}

impl RenameRequest {
    pub fn get_id(&self) -> &RequestId {
        &self.base.id
    }

    pub fn get_document_uri(&self) -> &String {
        &self.params.text_document_position.text_document.uri
    }

    pub fn get_position(&self) -> &Position {
        &self.params.text_document_position.position
    }

    pub fn get_new_name(&self) -> &String {
        &self.params.new_name
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct RenameParams {
    #[serde(flatten)]
    text_document_position: TextDocumentPositionParams,
    /// The new name of the symbol. If the given name is not valid the
    /// request must return a `ResponseError` with an
    /// appropriate message set.
    new_name: String,
    // WARNING: This is not to spec, this could also inherit WorkDoneProgressParams.
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RenameResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: Option<WorkspaceEdit>,
}

impl LspMessage for RenameResponse {}

impl RenameResponse {
    pub fn new(id: &RequestId) -> Self {
        RenameResponse {
            base: ResponseMessageBase::success(id),
            result: None,
        }
    }

    pub fn set_edit(&mut self, edit: WorkspaceEdit) {
        self.result = Some(edit);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::server::lsp::{
        WorkspaceEdit,
        messages::{textdocument_rename::RenameParams, utils::TextDocumentPositionParams},
        rpc::{Message, RequestId, RequestMessageBase},
        textdocument::{Position, Range, TextDocumentIdentifier, TextEdit},
    };

    use super::{RenameRequest, RenameResponse};

    #[test]
    fn deserialize() {
        let message = br#"{"params":{"textDocument":{"uri":"file:///dings"},"position":{"character":42,"line":3},"newName":"?dongs"},"method":"textDocument/rename","id":2,"jsonrpc":"2.0"}"#;
        let rename_request: RenameRequest = serde_json::from_slice(message).unwrap();

        assert_eq!(
            rename_request,
            RenameRequest {
                base: RequestMessageBase {
                    base: Message {
                        jsonrpc: "2.0".to_string(),
                    },
                    method: "textDocument/rename".to_string(),
                    id: RequestId::Integer(2)
                },
                params: RenameParams {
                    text_document_position: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier {
                            uri: "file:///dings".to_string()
                        },
                        position: Position::new(3, 42)
                    },
                    new_name: "?dongs".to_string()
                }
            }
        )
    }

    #[test]
    fn serialize() {
        let mut rename_response = RenameResponse::new(&RequestId::Integer(42));
        rename_response.set_edit(WorkspaceEdit {
            changes: Some(HashMap::from([(
                "file:///dings".to_string(),
                vec![TextEdit::new(Range::new(3, 42, 3, 47), "?dongs")],
            )])),
        });
        let expected_message = r#"{"jsonrpc":"2.0","id":42,"result":{"changes":{"file:///dings":[{"range":{"start":{"line":3,"character":42},"end":{"line":3,"character":47}},"newText":"?dongs"}]}}}"#;
        assert_eq!(
            serde_json::to_string(&rename_response).unwrap(),
            expected_message
        );
    }
}
