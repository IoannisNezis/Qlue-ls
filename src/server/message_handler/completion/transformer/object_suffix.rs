use ll_sparql_parser::syntax_kind::SyntaxKind;

use crate::server::{
    Server,
    lsp::{Command, CompletionList, InsertTextFormat, InsertTextMode, ItemDefaults},
    message_handler::indent::brace_nesting_depth,
};

use super::super::environment::{CompletionEnvironment, CompletionLocation};
use super::CompletionTransformer;

/// Transforms object completions to append a trailing ` .\n` with proper
/// indentation and cursor positioning.
///
/// This transformer adds a snippet suffix that completes the triple with a dot
/// and positions the cursor on a new line, ready for the next triple.
///
/// If a separator (`.`/`;`/`,`) already follows the cursor, the suffix is
/// omitted to avoid producing invalid syntax. In that case the transformer only
/// strips the trailing separator space the handlers append, so the completion
/// sits flush against the existing separator.
pub struct ObjectSuffixTransformer {
    indent: String,
    /// Whether a separator (`.`/`;`/`,`) already follows right after the cursor.
    followed_by_separator: bool,
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

        let followed_by_separator = matches!(
            env.following_kind,
            Some(SyntaxKind::Dot | SyntaxKind::Semicolon | SyntaxKind::Comma)
        );
        let indent = " "
            .repeat(brace_nesting_depth(env.anchor_token.as_ref()?))
            .repeat(server.settings.format.tab_size.unwrap_or(2) as usize);
        Some(Self {
            indent,
            followed_by_separator,
        })
    }
}

impl CompletionTransformer for ObjectSuffixTransformer {
    fn transform(&self, list: &mut CompletionList) {
        // A separator (`.`/`;`/`,`) already follows: don't append a suffix, just
        // drop the trailing separator space so the value sits flush against it.
        if self.followed_by_separator {
            for item in list.items.iter_mut() {
                if let Some(ref mut text_edit) = item.text_edit {
                    text_edit.new_text = text_edit.new_text.trim_end().to_string();
                }
                if let Some(ref mut insert_text) = item.insert_text {
                    *insert_text = insert_text.trim_end().to_string();
                }
            }
            return;
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::lsp::{
        CompletionItem, CompletionItemKind,
        textdocument::{Position, Range, TextEdit},
    };

    fn item_with_insert_text(insert_text: &str) -> CompletionItem {
        CompletionItem::new(
            "label",
            None,
            None,
            insert_text,
            CompletionItemKind::Variable,
            None,
        )
    }

    fn item_with_text_edit(new_text: &str) -> CompletionItem {
        let mut item =
            CompletionItem::new("label", None, None, "", CompletionItemKind::Variable, None);
        item.insert_text = None;
        item.text_edit = Some(TextEdit {
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 0),
            },
            new_text: new_text.to_string(),
        });
        item
    }

    fn list_with(items: Vec<CompletionItem>) -> CompletionList {
        CompletionList {
            is_incomplete: false,
            item_defaults: None,
            items,
        }
    }

    #[test]
    fn rewrites_insert_text_with_suffix() {
        let transformer = ObjectSuffixTransformer {
            indent: "  ".to_string(),
            followed_by_separator: false,
        };
        let mut list = list_with(vec![item_with_insert_text("?foo")]);

        transformer.transform(&mut list);

        assert_eq!(list.items[0].insert_text.as_deref(), Some("?foo .\n  $0"));
    }

    #[test]
    fn rewrites_text_edit_with_suffix() {
        let transformer = ObjectSuffixTransformer {
            indent: "    ".to_string(),
            followed_by_separator: false,
        };
        let mut list = list_with(vec![item_with_text_edit("wd:Q42")]);

        transformer.transform(&mut list);

        assert_eq!(
            list.items[0].text_edit.as_ref().unwrap().new_text,
            "wd:Q42 .\n    $0"
        );
    }

    #[test]
    fn trims_trailing_whitespace_before_suffix() {
        let transformer = ObjectSuffixTransformer {
            indent: String::new(),
            followed_by_separator: false,
        };
        let mut list = list_with(vec![item_with_insert_text("?foo   ")]);

        transformer.transform(&mut list);

        assert_eq!(list.items[0].insert_text.as_deref(), Some("?foo .\n$0"));
    }

    #[test]
    fn empty_indent_produces_no_leading_spaces() {
        let transformer = ObjectSuffixTransformer {
            indent: String::new(),
            followed_by_separator: false,
        };
        let mut list = list_with(vec![item_with_insert_text("?foo")]);

        transformer.transform(&mut list);

        assert_eq!(list.items[0].insert_text.as_deref(), Some("?foo .\n$0"));
    }

    #[test]
    fn sets_snippet_format_and_trigger_command() {
        let transformer = ObjectSuffixTransformer {
            indent: "  ".to_string(),
            followed_by_separator: false,
        };
        let mut list = list_with(vec![item_with_insert_text("?foo")]);

        transformer.transform(&mut list);

        let item = &list.items[0];
        assert_eq!(item.insert_text_format, Some(InsertTextFormat::Snippet));
        let command = item.command.as_ref().unwrap();
        assert_eq!(command.command, "triggerNewCompletion");
    }

    #[test]
    fn sets_item_defaults_to_insert_as_is() {
        let transformer = ObjectSuffixTransformer {
            indent: "  ".to_string(),
            followed_by_separator: false,
        };
        let mut list = list_with(vec![item_with_insert_text("?foo")]);

        transformer.transform(&mut list);

        let defaults = list.item_defaults.expect("item_defaults should be set");
        assert_eq!(defaults.insert_text_mode, Some(InsertTextMode::AsIs));
    }

    #[test]
    fn rewrites_both_insert_text_and_text_edit() {
        let transformer = ObjectSuffixTransformer {
            indent: "  ".to_string(),
            followed_by_separator: false,
        };
        let mut item = item_with_text_edit("wd:Q42");
        item.insert_text = Some("?foo".to_string());
        let mut list = list_with(vec![item]);

        transformer.transform(&mut list);

        assert_eq!(
            list.items[0].text_edit.as_ref().unwrap().new_text,
            "wd:Q42 .\n  $0"
        );
        assert_eq!(list.items[0].insert_text.as_deref(), Some("?foo .\n  $0"));
    }

    #[test]
    fn transforms_all_items() {
        let transformer = ObjectSuffixTransformer {
            indent: String::new(),
            followed_by_separator: false,
        };
        let mut list = list_with(vec![
            item_with_insert_text("?a"),
            item_with_insert_text("?b"),
        ]);

        transformer.transform(&mut list);

        assert_eq!(list.items[0].insert_text.as_deref(), Some("?a .\n$0"));
        assert_eq!(list.items[1].insert_text.as_deref(), Some("?b .\n$0"));
    }

    #[test]
    fn empty_list_is_noop_for_items() {
        let transformer = ObjectSuffixTransformer {
            indent: "  ".to_string(),
            followed_by_separator: false,
        };
        let mut list = list_with(vec![]);

        transformer.transform(&mut list);

        assert!(list.items.is_empty());
        // item_defaults is still set even with no items
        assert!(list.item_defaults.is_some());
    }

    #[test]
    fn separator_trims_trailing_space_on_insert_text() {
        let transformer = ObjectSuffixTransformer {
            indent: "  ".to_string(),
            followed_by_separator: true,
        };
        let mut list = list_with(vec![item_with_insert_text("?foo ")]);

        transformer.transform(&mut list);

        assert_eq!(list.items[0].insert_text.as_deref(), Some("?foo"));
    }

    #[test]
    fn separator_trims_trailing_space_on_text_edit() {
        let transformer = ObjectSuffixTransformer {
            indent: "  ".to_string(),
            followed_by_separator: true,
        };
        let mut list = list_with(vec![item_with_text_edit("wd:Q42 ")]);

        transformer.transform(&mut list);

        assert_eq!(list.items[0].text_edit.as_ref().unwrap().new_text, "wd:Q42");
    }

    #[test]
    fn separator_does_not_append_suffix_or_snippet() {
        let transformer = ObjectSuffixTransformer {
            indent: "  ".to_string(),
            followed_by_separator: true,
        };
        let mut list = list_with(vec![item_with_insert_text("?foo ")]);

        transformer.transform(&mut list);

        let item = &list.items[0];
        assert!(item.insert_text_format.is_none());
        assert!(item.command.is_none());
        assert!(list.item_defaults.is_none());
    }
}
