use serde::Serialize;

use crate::server::lsp::base_types::LSPAny;

#[derive(Debug, Serialize, PartialEq)]
pub struct Command {
    /// Title of the command, like `save`.
    pub title: String,
    /// The identifier of the actual command handler.
    pub command: String,
    /// Arguments that the command handler should be invoked with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<LSPAny>>,
}
