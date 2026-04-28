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
                    message: match &error {
                        ExecuteOperationErrorData::QLeverException(_) => "Qlever threw an error.",
                        ExecuteOperationErrorData::Connection(_) => {
                            "The connection to the endpoint failed."
                        }
                        ExecuteOperationErrorData::Canceled(_) => "The query was canceled.",
                        ExecuteOperationErrorData::InvalidFormat {
                            query: _,
                            message: _,
                        } => "Update result could not be deserialized.",
                        ExecuteOperationErrorData::Deserialization {
                            query: _,
                            message: _,
                        } => "The result of the query could not be deserialized.",
                        ExecuteOperationErrorData::Unknown => "An unknown error occured",
                    }
                    .to_string(),
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
    UpdateResult(ExecuteUpdateResponseResult),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteQueryResponseResult {
    pub time_ms: u128,
    pub result: Option<SparqlResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteUpdateResponseResult {
    pub operations: Vec<ExecuteUpdateOperationResult>,
    pub time: ExecuteUpdateGlobalTimeInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteUpdateGlobalTimeInfo {
    pub total: u64,
    pub parsing: u64,
    pub waiting_for_update_thread: u64,
    pub acquiring_delta_triples_write_lock: u64,
    pub operations: u64,
    pub metadata_update_for_snapshot: u64,
    pub disk_writeback: u64,
    pub snapshot_creation: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteUpdateOperationResult {
    pub status: String,
    #[serde(rename(deserialize = "delta-triples", serialize = "deltaTriples"))]
    pub delta_triples: DeltaTiples,
    #[serde(rename(deserialize = "located-triples", serialize = "locatedTriples"))]
    pub located_triples: LocatedTriplesStats,
    pub time: ExecuteUpdateOperationTimeInfo,
    pub update: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteUpdateOperationTimeInfo {
    pub execution: ExecutionTime,
    pub planning: u64,
    pub total: u64,
    pub update_metadata: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionTime {
    pub clear_cache: u64,
    pub compute_ids: ComputeIdsTime,
    pub delete_triples: TripleWriteTime,
    pub evaluate_where: u64,
    pub insert_triples: TripleWriteTime,
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeIdsTime {
    pub deduplication: u64,
    pub result_interpolation: u64,
    pub total: u64,
    pub vocab_lookup: u64,
}

// INFO: QLever reports `0` when the operation does not perform that kind of
// write (e.g. `insertTriples` is a plain integer for a DELETE WHERE), and a
// detailed breakdown otherwise.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TripleWriteTime {
    Skipped(u64),
    Detailed(TripleWriteTimeDetails),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripleWriteTimeDetails {
    pub external_permutation: ExternalPermutationTime,
    pub internal_permutation: InternalPermutationTime,
    pub make_internal_triples: u64,
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalPermutationTime {
    pub located_and_add: ExternalLocatedAndAdd,
    pub mark_triples: u64,
    pub remove_existing_triples: u64,
    pub remove_inverse_triples: u64,
    pub rewrite_local_vocab_entries: u64,
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalLocatedAndAdd {
    #[serde(rename(deserialize = "SPO"))]
    pub spo: PermutationLocateAdd,
    #[serde(rename(deserialize = "SOP"))]
    pub sop: PermutationLocateAdd,
    #[serde(rename(deserialize = "PSO"))]
    pub pso: PermutationLocateAdd,
    #[serde(rename(deserialize = "POS"))]
    pub pos: PermutationLocateAdd,
    #[serde(rename(deserialize = "OSP"))]
    pub osp: PermutationLocateAdd,
    #[serde(rename(deserialize = "OPS"))]
    pub ops: PermutationLocateAdd,
    pub total: u64,
    pub transform_handles: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalPermutationTime {
    pub located_and_add: InternalLocatedAndAdd,
    pub mark_triples: u64,
    pub remove_existing_triples: u64,
    pub remove_inverse_triples: u64,
    pub rewrite_local_vocab_entries: u64,
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalLocatedAndAdd {
    #[serde(rename(deserialize = "PSO"))]
    pub pso: PermutationLocateAdd,
    #[serde(rename(deserialize = "POS"))]
    pub pos: PermutationLocateAdd,
    pub total: u64,
    pub transform_handles: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermutationLocateAdd {
    pub add_to_located_triples: AddToLocatedTriples,
    pub locate_triples: u64,
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddToLocatedTriples {
    pub adding: u64,
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeltaTiples {
    pub before: TripleDelta,
    pub after: TripleDelta,
    pub difference: TripleDelta,
    pub operation: TripleDelta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocatedTriplesStats {
    #[serde(rename(deserialize = "SPO"))]
    pub spo: LocatedTriplesPermutationStats,
    #[serde(rename(deserialize = "SOP"))]
    pub sop: LocatedTriplesPermutationStats,
    #[serde(rename(deserialize = "PSO"))]
    pub pso: LocatedTriplesPermutationStats,
    #[serde(rename(deserialize = "POS"))]
    pub pos: LocatedTriplesPermutationStats,
    #[serde(rename(deserialize = "OSP"))]
    pub osp: LocatedTriplesPermutationStats,
    #[serde(rename(deserialize = "OPS"))]
    pub ops: LocatedTriplesPermutationStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocatedTriplesPermutationStats {
    #[serde(rename(deserialize = "blocks-affected", serialize = "blocksAffected"))]
    pub blocks_affected: u32,
    #[serde(rename(deserialize = "blocks-total", serialize = "blocksTotal"))]
    pub blocks_total: u32,
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
    Deserialization { query: String, message: String },
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
        QLeverStatus,
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
    fn deserialize_update_result() {
        let message = r#"{
    "operations": [
        {
            "delta-triples": {
                "after": {
                    "deleted": 0,
                    "inserted": 1,
                    "total": 1
                },
                "before": {
                    "deleted": 0,
                    "inserted": 1,
                    "total": 1
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
            "status": "OK",
            "time": {
                "execution": {
                    "clearCache": 0,
                    "computeIds": {
                        "deduplication": 0,
                        "resultInterpolation": 0,
                        "total": 0,
                        "vocabLookup": 0
                    },
                    "deleteTriples": 0,
                    "evaluateWhere": 0,
                    "insertTriples": {
                        "externalPermutation": {
                            "locatedAndAdd": {
                                "OPS": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "OSP": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "POS": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "PSO": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "SOP": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "SPO": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "total": 0,
                                "transformHandles": 0
                            },
                            "markTriples": 0,
                            "removeExistingTriples": 0,
                            "removeInverseTriples": 0,
                            "rewriteLocalVocabEntries": 0,
                            "total": 0
                        },
                        "internalPermutation": {
                            "locatedAndAdd": {
                                "POS": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "PSO": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "total": 0,
                                "transformHandles": 0
                            },
                            "markTriples": 0,
                            "removeExistingTriples": 0,
                            "removeInverseTriples": 0,
                            "rewriteLocalVocabEntries": 0,
                            "total": 0
                        },
                        "makeInternalTriples": 0,
                        "total": 0
                    },
                    "total": 0
                },
                "planning": 0,
                "total": 0,
                "updateMetadata": 0
            },
            "update": "INSERT DATA {\n  <a> <b> <b>\n}",
            "warnings": [
                "SPARQL 1.1 Update for QLever is experimental."
            ]
        }
    ],
    "time": {
        "total": 1,
        "parsing": 0,
        "waitingForUpdateThread": 0,
        "acquiringDeltaTriplesWriteLock": 0,
        "operations": 1,
        "metadataUpdateForSnapshot": 0,
        "diskWriteback": 0,
        "snapshotCreation": 0
    }
}"#;
        let _x: ExecuteUpdateResponseResult = serde_json::from_str(message).unwrap();
    }

    #[test]
    fn deserialize_update_result_delete_where() {
        // INFO: For a DELETE WHERE the shapes of `deleteTriples` and
        // `insertTriples` are flipped vs an INSERT: the detailed breakdown
        // shows up under `deleteTriples`, while `insertTriples` is a plain `0`.
        let message = r#"{
    "operations": [
        {
            "delta-triples": {
                "after": {
                    "deleted": 3,
                    "inserted": 1,
                    "total": 4
                },
                "before": {
                    "deleted": 2,
                    "inserted": 2,
                    "total": 4
                },
                "difference": {
                    "deleted": 1,
                    "inserted": -1,
                    "total": 0
                },
                "operation": {
                    "deleted": 1,
                    "inserted": 0,
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
            "status": "OK",
            "time": {
                "execution": {
                    "clearCache": 0,
                    "computeIds": {
                        "deduplication": 0,
                        "resultInterpolation": 0,
                        "total": 0,
                        "vocabLookup": 0
                    },
                    "deleteTriples": {
                        "externalPermutation": {
                            "locatedAndAdd": {
                                "OPS": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "OSP": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "POS": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "PSO": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "SOP": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "SPO": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "total": 0,
                                "transformHandles": 0
                            },
                            "markTriples": 0,
                            "removeExistingTriples": 0,
                            "removeInverseTriples": 0,
                            "rewriteLocalVocabEntries": 0,
                            "total": 0
                        },
                        "internalPermutation": {
                            "locatedAndAdd": {
                                "POS": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "PSO": {
                                    "addToLocatedTriples": {
                                        "adding": 0,
                                        "total": 0
                                    },
                                    "locateTriples": 0,
                                    "total": 0
                                },
                                "total": 0,
                                "transformHandles": 0
                            },
                            "markTriples": 0,
                            "removeExistingTriples": 0,
                            "removeInverseTriples": 0,
                            "rewriteLocalVocabEntries": 0,
                            "total": 0
                        },
                        "makeInternalTriples": 0,
                        "total": 0
                    },
                    "evaluateWhere": 0,
                    "insertTriples": 0,
                    "total": 0
                },
                "planning": 0,
                "total": 0,
                "updateMetadata": 0
            },
            "update": "DELETE WHERE {\n  <a> ?p ?o\n}",
            "warnings": [
                "SPARQL 1.1 Update for QLever is experimental."
            ]
        }
    ],
    "time": {
        "total": 1,
        "parsing": 0,
        "waitingForUpdateThread": 0,
        "acquiringDeltaTriplesWriteLock": 0,
        "operations": 1,
        "metadataUpdateForSnapshot": 0,
        "diskWriteback": 0,
        "snapshotCreation": 0
    }
}"#;
        let _x: ExecuteUpdateResponseResult = serde_json::from_str(message).unwrap();
    }
}
