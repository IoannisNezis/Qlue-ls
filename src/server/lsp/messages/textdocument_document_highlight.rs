use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::server::lsp::{
    LspMessage,
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::{Position, Range},
};

use super::utils::TextDocumentPositionParams;

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentHighlight
#[derive(Debug, Deserialize, PartialEq)]
pub struct DocumentHighlightRequest {
    #[serde(flatten)]
    base: RequestMessageBase,
    params: DocumentHighlightParams,
}

impl LspMessage for DocumentHighlightRequest {}

impl DocumentHighlightRequest {
    pub fn get_id(&self) -> &RequestId {
        &self.base.id
    }

    pub fn get_document_uri(&self) -> &String {
        &self.params.text_document_position.text_document.uri
    }

    pub fn get_position(&self) -> &Position {
        &self.params.text_document_position.position
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct DocumentHighlightParams {
    #[serde(flatten)]
    text_document_position: TextDocumentPositionParams,
    // WARNING: This is not to spec, this could also inherit
    // WorkDoneProgressParams and PartialResultParams.
}

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#documentHighlight
#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct DocumentHighlight {
    /// The range this highlight applies to.
    pub range: Range,
    /// The highlight kind, default is `DocumentHighlightKind::Text`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<DocumentHighlightKind>,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Clone)]
#[repr(u8)]
pub enum DocumentHighlightKind {
    /// A textual occurrence.
    Text = 1,
    /// Read-access of a symbol, like reading a variable.
    Read = 2,
    /// Write-access of a symbol, like writing to a variable.
    Write = 3,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DocumentHighlightResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: Option<Vec<DocumentHighlight>>,
}

impl LspMessage for DocumentHighlightResponse {}

impl DocumentHighlightResponse {
    pub fn new(id: &RequestId) -> Self {
        DocumentHighlightResponse {
            base: ResponseMessageBase::success(id),
            result: None,
        }
    }

    pub fn set_highlights(&mut self, highlights: Vec<DocumentHighlight>) {
        self.result = Some(highlights);
    }
}

#[cfg(test)]
mod tests {
    use crate::server::lsp::{
        messages::{
            textdocument_document_highlight::DocumentHighlightParams,
            utils::TextDocumentPositionParams,
        },
        rpc::{Message, RequestId, RequestMessageBase},
        textdocument::{Position, Range, TextDocumentIdentifier},
    };

    use super::{
        DocumentHighlight, DocumentHighlightKind, DocumentHighlightRequest,
        DocumentHighlightResponse,
    };

    #[test]
    fn deserialize() {
        let message = br#"{"params":{"textDocument":{"uri":"file:///dings"},"position":{"character":42,"line":3}},"method":"textDocument/documentHighlight","id":2,"jsonrpc":"2.0"}"#;
        let highlight_request: DocumentHighlightRequest = serde_json::from_slice(message).unwrap();

        assert_eq!(
            highlight_request,
            DocumentHighlightRequest {
                base: RequestMessageBase {
                    base: Message {
                        jsonrpc: "2.0".to_string(),
                    },
                    method: "textDocument/documentHighlight".to_string(),
                    id: RequestId::Integer(2)
                },
                params: DocumentHighlightParams {
                    text_document_position: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier {
                            uri: "file:///dings".to_string()
                        },
                        position: Position::new(3, 42)
                    },
                }
            }
        )
    }

    #[test]
    fn serialize() {
        let mut highlight_response = DocumentHighlightResponse::new(&RequestId::Integer(42));
        highlight_response.set_highlights(vec![
            DocumentHighlight {
                range: Range::new(3, 42, 3, 47),
                kind: Some(DocumentHighlightKind::Read),
            },
            DocumentHighlight {
                range: Range::new(5, 0, 5, 5),
                kind: None,
            },
        ]);
        let expected_message = r#"{"jsonrpc":"2.0","id":42,"result":[{"range":{"start":{"line":3,"character":42},"end":{"line":3,"character":47}},"kind":2},{"range":{"start":{"line":5,"character":0},"end":{"line":5,"character":5}}}]}"#;
        assert_eq!(
            serde_json::to_string(&highlight_response).unwrap(),
            expected_message
        );
    }
}
