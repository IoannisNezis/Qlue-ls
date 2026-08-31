use serde::Serialize;

use crate::server::lsp::{LspMessage, rpc::NotificationMessageBase};

/// Reports a completion query that was just rendered and (attempted to be) executed.
///
/// Purely diagnostic: it lets the client show the rendered query, its result size and its
/// failure reason while a completion template is being edited.
#[derive(Debug, Serialize)]
pub struct CompletionQueryNotification {
    #[serde(flatten)]
    pub base: NotificationMessageBase,
    pub params: CompletionQueryParams,
}

impl CompletionQueryNotification {
    pub(crate) fn new(params: CompletionQueryParams) -> Self {
        Self {
            base: NotificationMessageBase::new("qlueLs/completionQuery"),
            params,
        }
    }
}

impl LspMessage for CompletionQueryNotification {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionQueryParams {
    /// Template name, as registered in tera: `"{backend}-{template}"`.
    pub template: String,
    /// The rendered query. Empty if rendering the template failed.
    pub query: String,
    pub url: String,
    pub duration_ms: u32,
    /// Number of bindings the endpoint returned, before search-term filtering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
