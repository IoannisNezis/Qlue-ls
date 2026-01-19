use serde::{Deserialize, Serialize};

use crate::server::lsp::{
    LspMessage,
    rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
    textdocument::TextDocumentIdentifier,
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct FoldingRangeRequest {
    #[serde(flatten)]
    base: RequestMessageBase,
    params: FoldingRangeParams,
}
impl FoldingRangeRequest {
    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }
    pub(crate) fn get_document_uri(&self) -> &String {
        &self.params.text_document.uri
    }
}

impl LspMessage for FoldingRangeRequest {}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct FoldingRangeParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FoldingRangeResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    result: Option<Vec<FoldingRange>>,
}

impl LspMessage for FoldingRangeResponse {}

impl FoldingRangeResponse {
    pub fn new(id: &RequestId) -> Self {
        FoldingRangeResponse {
            base: ResponseMessageBase::success(id),
            result: None,
        }
    }
    pub fn set_result(&mut self, folding_ranges: Vec<FoldingRange>) {
        self.result = Some(folding_ranges);
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FoldingRange {
    /**
     * The zero-based start line of the range to fold. The folded area starts
     * after the line's last character. To be valid, the end must be zero or
     * larger and smaller than the number of lines in the document.
     */
    pub start_line: u32,

    /**
     * The zero-based character offset from where the folded range starts. If
     * not defined, defaults to the length of the start line.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_character: Option<u32>,

    /**
     * The zero-based end line of the range to fold. The folded area ends with
     * the line's last character. To be valid, the end must be zero or larger
     * and smaller than the number of lines in the document.
     */
    pub end_line: u32,

    /**
     * The zero-based character offset before the folded range ends. If not
     * defined, defaults to the length of the end line.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_character: Option<u32>,

    /**
     * Describes the kind of the folding range such as `comment` or `region`.
     * The kind is used to categorize folding ranges and used by commands like
     * 'Fold all comments'. See [FoldingRangeKind](#FoldingRangeKind) for an
     * enumeration of standardized kinds.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<FoldingRangeKind>,

    /**
     * The text that the client should show when the specified range is
     * collapsed. If not defined or not supported by the client, a default
     * will be chosen by the client.
     *
     * @since 3.17.0 - proposed
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapsed_text: Option<String>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FoldingRangeKind {
    //Comment,
    /**
     * Folding range for imports or includes
     */
    Imports,
    //Region,
}

#[cfg(test)]
mod tests {

    use crate::server::lsp::{
        FoldingRangeKind, FoldingRangeRequest, FoldingRangeResponse,
        messages::textdocument_folding_range::{FoldingRange, FoldingRangeParams},
        rpc::{Message, RequestId, RequestMessageBase},
        textdocument::TextDocumentIdentifier,
    };

    #[test]
    fn deserialize_folding_range_request() {
        let message = br#"{
                            "params": {
                                "textDocument": {
                                    "uri": "file:///example.sparql"
                                }
                            },
                            "method": "textDocument/foldingRange",
                            "id": 5,
                            "jsonrpc": "2.0"
                          }"#;

        let folding_request: FoldingRangeRequest = serde_json::from_slice(message).unwrap();

        assert_eq!(
            folding_request,
            FoldingRangeRequest {
                base: RequestMessageBase {
                    base: Message {
                        jsonrpc: "2.0".to_string(),
                    },
                    method: "textDocument/foldingRange".to_string(),
                    id: RequestId::Integer(5)
                },
                params: FoldingRangeParams {
                    text_document: TextDocumentIdentifier {
                        uri: "file:///example.sparql".to_string()
                    },
                }
            }
        )
    }

    #[test]
    fn serialize_folding_range_response() {
        let mut folding_response = FoldingRangeResponse::new(&RequestId::Integer(5));

        // Pretend we found a prologue block from line 0–2
        folding_response.result = Some(vec![FoldingRange {
            start_line: 0,
            start_character: None,
            end_line: 2,
            end_character: None,
            kind: Some(FoldingRangeKind::Imports),
            collapsed_text: None,
        }]);

        let expected_message = r#"{
        "jsonrpc":"2.0",
        "id":5,
        "result":[
            {"startLine":0,"endLine":2,"kind":"imports"}
        ]
    }"#;

        assert_eq!(
            serde_json::to_string(&folding_response).unwrap(),
            expected_message.replace(char::is_whitespace, "")
        );
    }
}
