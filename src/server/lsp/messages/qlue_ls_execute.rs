#[cfg(target_arch = "wasm32")]
use crate::server::lsp::rpc::NotificationMessageBase;
use crate::{
    server::{
        lsp::{
            LspMessage,
            errors::{ErrorCode, LSPErrorBase},
            rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
            textdocument::TextDocumentIdentifier,
        },
        sparql_operations::ConnectionError,
    },
    sparql::results::SparqlResult,
};
#[cfg(target_arch = "wasm32")]
use lazy_sparql_result_reader::parser::PartialResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ExecuteOperationRequest {
    #[serde(flatten)]
    base: RequestMessageBase,
    pub params: ExecuteOperationParams,
}
impl ExecuteOperationRequest {
    pub(crate) fn get_id(&self) -> &RequestId {
        &self.base.id
    }
}

impl LspMessage for ExecuteOperationRequest {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteOperationParams {
    pub text_document: TextDocumentIdentifier,
    pub max_result_size: Option<usize>,
    pub result_offset: Option<usize>,
    pub query_id: Option<String>,
    pub lazy: Option<bool>,
    pub access_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecuteOperationResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ExecuteOperationResponseResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ExecuteOperationError>,
}

impl ExecuteOperationResponse {
    pub(crate) fn success(id: &RequestId, result: ExecuteOperationResponseResult) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: Some(result),
            error: None,
        }
    }

    pub(crate) fn error(id: &RequestId, error: ExecuteOperationErrorData) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: None,
            error: Some(ExecuteOperationError {
                base: LSPErrorBase {
                    code: ErrorCode::RequestFailed,
                    message: "The Query was rejected by the SPARQL endpoint".to_string(),
                },
                data: error,
            }),
        }
    }
}

impl LspMessage for ExecuteOperationResponse {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExecuteOperationResponseResult {
    QueryResult(ExecuteQueryResponseResult),
    UpdateResult(Vec<ExecuteUpdateResponseResult>),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteQueryResponseResult {
    pub time_ms: u128,
    pub result: Option<SparqlResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteUpdateResponseResult {
    pub status: String,
    #[serde(rename(deserialize = "delta-triples", serialize = "deltaTriples"))]
    pub delta_triples: DeltaTiples,
    pub time: TimeInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeInfo {
    #[serde(deserialize_with = "deserialize_ms")]
    total: u64,
    #[serde(deserialize_with = "deserialize_ms")]
    planning: u64,
    #[serde(rename = "where", deserialize_with = "deserialize_ms")]
    lookup: u64,
    update: UpdateTiming,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTiming {
    #[serde(deserialize_with = "deserialize_ms")]
    total: u64,
    #[serde(deserialize_with = "deserialize_ms")]
    preparation: u64,
    #[serde(deserialize_with = "deserialize_ms")]
    delete: u64,
    #[serde(deserialize_with = "deserialize_ms")]
    insert: u64,
}

use serde::Deserializer;
use serde::de;

fn deserialize_ms<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let ms = s
        .strip_suffix("ms")
        .ok_or_else(|| de::Error::custom("expected value ending with 'ms'"))?;
    ms.parse::<u64>().map_err(de::Error::custom)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeltaTiples {
    pub before: TripleDelta,
    pub after: TripleDelta,
    pub difference: TripleDelta,
    pub operation: TripleDelta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TripleDelta {
    pub deleted: i64,
    pub inserted: i64,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteOperationError {
    #[serde(flatten)]
    base: LSPErrorBase,
    data: ExecuteOperationErrorData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExecuteOperationErrorData {
    QLeverException(QLeverException),
    Connection(ConnectionError),
    Canceled(CanceledError),
    InvalidFormat { query: String, message: String },
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanceledError {
    pub query: String,
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
        use lazy_sparql_result_reader::parser::PartialResult;

        Self {
            base: NotificationMessageBase::new("qlueLs/partialResult"),
            params: PartialResult::from(chunk),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl LspMessage for PartialSparqlResultNotification {}

#[cfg(test)]
mod test {
    use crate::server::lsp::{
        ExecuteOperationErrorData, ExecuteUpdateResponseResult, Metadata, QLeverException,
        QLeverStatus, TimeInfo,
    };

    #[test]
    fn serialize_execute_query_error() {
        let error = ExecuteOperationErrorData::QLeverException(QLeverException {
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

    #[test]
    fn deserialize_timing_info() {
        let message = r#"
        {
            "planning": "0ms",
            "total": "0ms",
            "where": "0ms",
            "update": {
                "delete": "0ms",
                "insert": "0ms",
                "preparation": "0ms",
                "total": "0ms"
            }
        }"#;
        let _x: TimeInfo = serde_json::from_str(message).unwrap();
    }

    #[test]
    fn deserialize_update_result() {
        let message = r#"[
    {
        "delta-triples": {
            "after": {
                "deleted": 1,
                "inserted": 3,
                "total": 4
            },
            "before": {
                "deleted": 1,
                "inserted": 3,
                "total": 4
            },
            "difference": {
                "deleted": 0,
                "inserted": 0,
                "total": 0
            },
            "operation": {
                "deleted": 0,
                "inserted": 1,
                "total": 1
            }
        },
        "located-triples": {
            "OPS": {
                "blocks-affected": 1,
                "blocks-total": 0
            },
            "OSP": {
                "blocks-affected": 1,
                "blocks-total": 0
            },
            "POS": {
                "blocks-affected": 1,
                "blocks-total": 0
            },
            "PSO": {
                "blocks-affected": 1,
                "blocks-total": 0
            },
            "SOP": {
                "blocks-affected": 1,
                "blocks-total": 0
            },
            "SPO": {
                "blocks-affected": 1,
                "blocks-total": 0
            }
        },
        "runtimeInformation": {
            "meta": {
                "time_query_planning": 0
            },
            "query_execution_tree": {
                "cache_status": "computed",
                "children": [],
                "column_names": [],
                "description": "NeutralElement",
                "details": null,
                "estimated_column_multiplicities": [],
                "estimated_operation_cost": 0,
                "estimated_size": 1,
                "estimated_total_cost": 0,
                "operation_time": 0,
                "original_operation_time": 0,
                "original_total_time": 0,
                "result_cols": 0,
                "result_rows": 1,
                "status": "fully materialized",
                "total_time": 0
            }
        },
        "status": "OK",
        "time": {
            "planning": "0ms",
            "total": "0ms",
            "update": {
                "delete": "0ms",
                "insert": "0ms",
                "preparation": "0ms",
                "total": "0ms"
            },
            "where": "0ms"
        },
        "update": "INSERT DATA {\n  <x> <y> <z>\n}",
        "warnings": [
            "SPARQL 1.1 Update for QLever is experimental."
        ]
    }
]"#;
        let _x: Vec<ExecuteUpdateResponseResult> = serde_json::from_str(message).unwrap();
    }
}
