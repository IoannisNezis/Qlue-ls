use crate::server::{
    Server,
    lsp::{Command, CompletionList, InsertTextFormat},
    state::ClientType,
};

use super::super::environment::{CompletionEnvironment, CompletionLocation};
use super::CompletionTransformer;

/// Transforms object completions to append a trailing ` .\n` with proper
/// indentation and cursor positioning.
///
/// This transformer adds a snippet suffix that completes the triple with a dot
/// and positions the cursor on a new line, ready for the next triple.
pub struct ObjectSuffixTransformer {
    indent: String,
}

impl ObjectSuffixTransformer {
    /// Creates a new ObjectSuffixTransformer if the environment is in Object position
    /// and the setting is enabled.
    ///
    /// Returns `None` if the transformation doesn't apply.
    pub(in crate::server::message_handler::completion) fn try_from_env(
        server: &Server,
        env: &CompletionEnvironment,
    ) -> Option<Self> {
        if !matches!(env.location, CompletionLocation::Object(_))
            || !server.settings.completion.object_completion_suffix
        {
            return None;
        }

        let indent = server
            .state
            .client_type
            .as_ref()
            .is_some_and(|client_type| matches!(client_type, ClientType::Monaco))
            .then_some(String::new())
            .unwrap_or_else(|| env.line_indentation.clone());

        Some(Self { indent })
    }
}

impl CompletionTransformer for ObjectSuffixTransformer {
    fn transform(&self, list: &mut CompletionList) {
        for item in list.items.iter_mut() {
            // Handle text_edit (used by online completions)
            if let Some(ref mut text_edit) = item.text_edit {
                text_edit.new_text =
                    format!("{} .\n{}$0", text_edit.new_text.trim_end(), self.indent);
            }
            // Handle insert_text (used by variable completions)
            if let Some(ref mut insert_text) = item.insert_text {
                *insert_text = format!("{} .\n{}$0", insert_text.trim_end(), self.indent);
            }
            item.insert_text_format = Some(InsertTextFormat::Snippet);
            item.command = Some(Command {
                title: "triggerNewCompletion".to_string(),
                command: "triggerNewCompletion".to_string(),
                arguments: None,
            });
        }
    }
}
