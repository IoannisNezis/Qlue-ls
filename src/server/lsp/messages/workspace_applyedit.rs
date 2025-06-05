use std::collections::HashMap;

use crate::server::lsp::{
    rpc::{RequestMessageBase, ResponseMessageBase},
    textdocument::TextEdit,
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

#[derive(Debug, Deserialize, PartialEq)]
pub struct ApplyWorkspaceEditResult {
    pub applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}
