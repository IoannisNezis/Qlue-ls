use ll_sparql_parser::{
    ast::{AstNode, TriplesBlock},
    syntax_kind::SyntaxKind,
};

use crate::server::{
    Server,
    lsp::{
        CompletionItemKind, CompletionList,
        textdocument::{Position, Range, TextEdit},
    },
};

use super::super::environment::{CompletionEnvironment, CompletionLocation};
use super::super::utils::get_replace_range;
use super::CompletionTransformer;

/// Transforms subject completions to use semicolon notation when the completed
/// subject matches the previous triple's subject.
///
/// This transformer replaces the trailing dot with a semicolon and positions
/// the cursor on a new indented line, allowing the user to continue adding
/// predicates to the same subject.
pub struct SemicolonTransformer {
    subject: String,
    dot_start: Position,
    indent: String,
    replace_range: Range,
}

impl SemicolonTransformer {
    /// Creates a new SemicolonTransformer if the environment is in Subject position
    /// and there's a previous triple with a trailing dot that can be converted.
    ///
    /// Returns `None` if the transformation doesn't apply.
    pub(in crate::server::message_handler::completion) fn try_from_env(
        server: &Server,
        env: &CompletionEnvironment,
    ) -> Option<Self> {
        if !matches!(env.location, CompletionLocation::Subject)
            || !server.settings.completion.same_subject_semicolon
        {
            return None;
        }
        let document_text = env.tree.text().to_string();
        let replace_range = get_replace_range(env);

        let triple_block = TriplesBlock::cast(env.anchor_token.as_ref()?.parent()?)?;
        let subject = triple_block.triples().first()?.subject()?;
        let dot_start = Position::from_byte_index(
            triple_block
                .syntax()
                .last_token()
                .and_then(|last| (last.kind() == SyntaxKind::Dot).then_some(last.text_range()))?
                .start(),
            &document_text,
        )?;
        let verb_start = triple_block
            .triples()
            .first()?
            .properties_list_path()?
            .properties()
            .first()?
            .verb
            .syntax()
            .text_range()
            .start()
            .into();
        let line_start = document_text[..verb_start]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        let indent = " ".repeat(verb_start - line_start);

        Some(Self {
            replace_range,
            subject: subject.text(),
            dot_start,
            indent,
        })
    }
}

impl CompletionTransformer for SemicolonTransformer {
    fn transform(&self, list: &mut CompletionList) {
        for item in list.items.iter_mut() {
            if item.kind != CompletionItemKind::Variable {
                continue;
            }
            let item_value = item
                .text_edit
                .as_ref()
                .map(|te| te.new_text.trim().to_string())
                .or_else(|| item.insert_text.as_ref().map(|s| s.trim().to_string()))
                .map(|insert_text| {
                    if item.kind == CompletionItemKind::Variable && !insert_text.starts_with("?") {
                        format!("?{}", insert_text)
                    } else {
                        insert_text
                    }
                })
                .unwrap_or_default();

            if item_value != self.subject {
                continue;
            }

            item.text_edit = Some(TextEdit {
                range: self.replace_range.clone(),
                new_text: String::new(),
            });
            // Main edit: clear the search term range
            item.insert_text = None;

            // Replace from dot start to replace_range start with semicolon + newline + indent
            // This removes the dot and any whitespace between it and the cursor position
            let dot_to_cursor_range = Range {
                start: self.dot_start.clone(),
                end: self.replace_range.start.clone(),
            };
            item.additional_text_edits = Some(vec![TextEdit {
                range: dot_to_cursor_range,
                new_text: format!(";\n{}", self.indent),
            }]);
            item.detail = Some("Continue with semicolon".to_string());
        }
    }
}
