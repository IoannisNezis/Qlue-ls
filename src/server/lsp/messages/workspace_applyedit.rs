use std::collections::HashMap;

use crate::server::lsp::{
    rpc::{RequestMessageBase, ResponseMessageBase},
    textdocument::TextEdit,
    LspMessage, RequestMarker, ResponseMarker,
};
use serde::{Deserialize, Serialize};

use super::WorkspaceEdit;

// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_applyEdit
#[derive(Debug, Serialize, PartialEq)]
pub struct WorkspaceEditRequest {
    #[serde(flatten)]
    pub base: RequestMessageBase,
    pub params: ApplyWorkspaceEditParams,
}

impl LspMessage for WorkspaceEditRequest {
    type Kind = RequestMarker;

    fn method(&self) -> Option<&str> {
        Some("workspace/applyEdit")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        Some(&self.base.id)
    }
}

impl WorkspaceEditRequest {
    pub fn new(id: u32, changes: HashMap<String, Vec<TextEdit>>) -> Self {
        Self {
            base: RequestMessageBase::new("workspace/applyEdit", id),
            params: ApplyWorkspaceEditParams {
                label: None,
                edit: WorkspaceEdit {
                    changes: Some(changes),
                },
            },
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ApplyWorkspaceEditParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub edit: WorkspaceEdit,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct WorkspaceEditResponse {
    #[serde(flatten)]
    base: ResponseMessageBase,
    pub result: Option<ApplyWorkspaceEditResult>,
}

impl LspMessage for WorkspaceEditResponse {
    type Kind = ResponseMarker;

    fn method(&self) -> Option<&str> {
        Some("workspace/applyEdit")
    }

    fn id(&self) -> Option<&crate::server::lsp::rpc::RequestId> {
        self.base.request_id()
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ApplyWorkspaceEditResult {
    pub applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}
