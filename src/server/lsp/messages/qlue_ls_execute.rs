use serde::{Deserialize, Serialize};

use crate::{
    server::lsp::{
        rpc::{RequestId, RequestMessageBase, ResponseMessageBase},
        textdocument::TextDocumentIdentifier,
        LspMessage, RequestMarker, ResponseMarker,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_result_size: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ExectuteQueryResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    pub result: Option<SparqlResult>,
}
impl ExectuteQueryResponse {
    pub(crate) fn success(id: &RequestId, result: SparqlResult) -> Self {
        Self {
            base: ResponseMessageBase::success(id),
            result: Some(result),
        }
    }
}

impl LspMessage for ExectuteQueryResponse {
    type Kind = ResponseMarker;

    fn method(&self) -> Option<&str> {
        None
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        self.base.id.request_id()
    }
}
