use serde::{Deserialize, Serialize};

use crate::{
    server::{
        fetch::ConnectionError,
        lsp::{
            errors::{ErrorCode, LSPErrorBase},
            rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
            textdocument::TextDocumentIdentifier,
            LspMessage, RequestMarker, ResponseMarker,
        },
    },
    sparql::results::SparqlResult,
};

#[derive(Debug, Deserialize)]
pub struct ExecuteQueryRequest {
    #[serde(flatten)]
    base: RequestMessageBase,
    pub params: ExecuteQueryParams,
}
impl ExecuteQueryRequest {
    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

impl LspMessage for ExecuteQueryRequest {
    type Kind = RequestMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/executeQuery")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        Some(&self.base.id)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteQueryParams {
    pub text_document: TextDocumentIdentifier,
    pub max_result_size: Option<u32>,
    pub result_offset: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ExecuteQueryResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<SparqlResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ExecuteQueryError>,
}
impl ExecuteQueryResponse {
    pub(crate) fn success(id: &RequestId, result: SparqlResult) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: Some(result),
            error: None,
        }
    }

    pub(crate) fn error(id: &RequestId, error: ExecuteQueryErrorData) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: None,
            error: Some(ExecuteQueryError {
                base: LSPErrorBase {
                    code: ErrorCode::RequestFailed,
                    message: "The Query was rejected by the SPARQL endpoint".to_string(),
                },
                data: error,
            }),
        }
    }
}

impl LspMessage for ExecuteQueryResponse {
    type Kind = ResponseMarker;

    fn method(&self) -> Option<&str> {
        None
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        self.base.id.request_id()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteQueryError {
    #[serde(flatten)]
    base: LSPErrorBase,
    data: ExecuteQueryErrorData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExecuteQueryErrorData {
    QLeverException(QLeverException),
    Connection(ConnectionError),
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QLeverException {
    pub exception: String,
    pub query: String,
    pub status: QLeverStatus,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    line: u32,
    position_in_line: u32,
    start_index: u32,
    stop_index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum QLeverStatus {
    #[serde(rename = "ERROR")]
    Error,
}

#[cfg(test)]
mod test {
    use crate::server::lsp::{ExecuteQueryErrorData, Metadata, QLeverException, QLeverStatus};

    #[test]
    fn serialize_execute_query_error() {
        let error = ExecuteQueryErrorData::QLeverException(QLeverException {
            exception: "foo".to_string(),
            query: "bar".to_string(),
            metadata: Metadata {
                line: 0,
                position_in_line: 0,
                start_index: 0,
                stop_index: 0,
            },
            status: QLeverStatus::Error,
        });
        let serialized = serde_json::to_string(&error).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"QLeverException","exception":"foo","query":"bar","status":"ERROR","metadata":{"line":0,"positionInLine":0,"startIndex":0,"stopIndex":0}}"#
        )
    }
}
