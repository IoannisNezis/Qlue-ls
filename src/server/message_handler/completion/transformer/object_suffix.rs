use ll_sparql_parser::{SyntaxToken, syntax_kind::SyntaxKind};

use crate::server::{
    Server,
    lsp::{Command, CompletionList, InsertTextFormat, InsertTextMode, ItemDefaults},
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
        let indent = " "
            .repeat(get_indentation(env.anchor_token.as_ref()?))
            .repeat(server.settings.format.tab_size.unwrap_or(2) as usize);
        Some(Self { indent })
    }
}

/// Returns the indentation level for a syntax token based on its nesting depth
/// within `GroupGraphPattern` nodes.
///
/// # Limitations
///
/// This function only considers `GroupGraphPattern` for indentation. Other
/// brace-delimited constructs like `QuadPattern`, `QuadData`, `ConstructTemplate`,
/// `QuadsNotTriples`, `InlineDataOneVar`, and `InlineDataFull` are not counted.
///
/// This is sufficient for tokens inside WHERE clauses and graph patterns, but
/// will not produce correct indentation for tokens in UPDATE data blocks or
/// CONSTRUCT templates.
fn get_indentation(syntax_token: &SyntaxToken) -> usize {
    syntax_token.parent_ancestors().fold(0, |acc, node| {
        if matches!(node.kind(), SyntaxKind::GroupGraphPattern) {
            acc + 1
        } else {
            acc
        }
    })
}

impl CompletionTransformer for ObjectSuffixTransformer {
    fn transform(&self, list: &mut CompletionList) {
        list.item_defaults = Some(ItemDefaults {
            commit_characters: None,
            edit_range: None,
            insert_text_format: None,
            insert_text_mode: Some(InsertTextMode::AsIs),
            data: None,
        });
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
