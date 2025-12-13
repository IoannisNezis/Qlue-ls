use crate::{
    server::{
        fetch::ConnectionError,
        lsp::{
            LspMessage, RequestMarker, ResponseMarker,
            errors::{ErrorCode, LSPErrorBase},
            rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
            textdocument::TextDocumentIdentifier,
        },
    },
    sparql::results::SparqlResult,
};
use serde::{Deserialize, Serialize};

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
    pub query_id: Option<String>,
    pub lazy: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ExecuteQueryResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ExecuteQueryResponseResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ExecuteQueryError>,
}

impl ExecuteQueryResponse {
    pub(crate) fn success(id: &RequestId, result: ExecuteQueryResponseResult) -> Self {
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteQueryResponseResult {
    pub time_ms: u128,
    pub result: Option<SparqlResult>,
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
    pub metadata: Option<Metadata>,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(target_arch = "wasm32")]
pub struct PartialSparqlResultNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: PartialResult,
}

#[cfg(target_arch = "wasm32")]
impl PartialSparqlResultNotification {
    pub(crate) fn new(chunk: PartialResult) -> Self {
        Self {
            base: NotificationMessageBase::new("qlueLs/partialResult"),
            params: PartialResult::from(chunk),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl LspMessage for PartialSparqlResultNotification {
    type Kind = NotificationMarker;

    fn method(&self) -> Option<&str> {
        Some("qlueLs/partialResult")
    }

    fn id(&self) -> Option<&RequestId> {
        None
    }
}

#[cfg(test)]
mod test {
    use crate::server::lsp::{ExecuteQueryErrorData, Metadata, QLeverException, QLeverStatus};

    #[test]
    fn serialize_execute_query_error() {
        let error = ExecuteQueryErrorData::QLeverException(QLeverException {
            exception: "foo".to_string(),
            query: "bar".to_string(),
            metadata: Some(Metadata {
                line: 0,
                position_in_line: 0,
                start_index: 0,
                stop_index: 0,
            }),
            status: QLeverStatus::Error,
        });
        let serialized = serde_json::to_string(&error).unwrap();
        assert_eq!(
            serialized,
            r#"{"type":"QLeverException","exception":"foo","query":"bar","status":"ERROR","metadata":{"line":0,"positionInLine":0,"startIndex":0,"stopIndex":0}}"#
        )
    }
}
