use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    LspMessage,
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::{Location, Position},
};

use super::utils::TextDocumentPositionParams;

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_references
#[derive(Debug, Deserialize, PartialEq)]
pub struct ReferencesRequest {
    #[serde(flatten)]
    base: RequestMessageBase,
    params: ReferenceParams,
}

impl LspMessage for ReferencesRequest {}

impl ReferencesRequest {
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
struct ReferenceParams {
    #[serde(flatten)]
    text_document_position: TextDocumentPositionParams,
    context: ReferenceContext,
    // WARNING: This is not to spec, this could also inherit
    // WorkDoneProgressParams and PartialResultParams.
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ReferenceContext {
    /// Include the declaration of the current symbol.
    include_declaration: bool,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ReferencesResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: Option<Vec<Location>>,
}

impl LspMessage for ReferencesResponse {}

impl ReferencesResponse {
    pub fn new(id: &RequestId) -> Self {
        ReferencesResponse {
            base: ResponseMessageBase::success(id),
            result: None,
        }
    }

    pub fn set_locations(&mut self, locations: Vec<Location>) {
        self.result = Some(locations);
    }
}

#[cfg(test)]
mod tests {
    use crate::server::lsp::{
        messages::{
            textdocument_references::{ReferenceContext, ReferenceParams},
            utils::TextDocumentPositionParams,
        },
        rpc::{Message, RequestId, RequestMessageBase},
        textdocument::{Location, Position, Range, TextDocumentIdentifier},
    };

    use super::{ReferencesRequest, ReferencesResponse};

    #[test]
    fn deserialize() {
        let message = br#"{"params":{"textDocument":{"uri":"file:///dings"},"position":{"character":42,"line":3},"context":{"includeDeclaration":true}},"method":"textDocument/references","id":2,"jsonrpc":"2.0"}"#;
        let references_request: ReferencesRequest = serde_json::from_slice(message).unwrap();

        assert_eq!(
            references_request,
            ReferencesRequest {
                base: RequestMessageBase {
                    base: Message {
                        jsonrpc: "2.0".to_string(),
                    },
                    method: "textDocument/references".to_string(),
                    id: RequestId::Integer(2)
                },
                params: ReferenceParams {
                    text_document_position: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier {
                            uri: "file:///dings".to_string()
                        },
                        position: Position::new(3, 42)
                    },
                    context: ReferenceContext {
                        include_declaration: true
                    }
                }
            }
        )
    }

    #[test]
    fn serialize() {
        let mut references_response = ReferencesResponse::new(&RequestId::Integer(42));
        references_response.set_locations(vec![Location {
            uri: "file:///dings".to_string(),
            range: Range::new(3, 42, 3, 47),
        }]);
        let expected_message = r#"{"jsonrpc":"2.0","id":42,"result":[{"uri":"file:///dings","range":{"start":{"line":3,"character":42},"end":{"line":3,"character":47}}}]}"#;
        assert_eq!(
            serde_json::to_string(&references_response).unwrap(),
            expected_message
        );
    }
}
