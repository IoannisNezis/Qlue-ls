use crate::server::{
    Server,
    lsp::{CompletionList, InsertTextFormat},
};

use super::CompletionTransformer;

/// Rewrites snippet completions as plain text for clients that don't announce
/// `completionItem.snippetSupport`.
///
/// Placeholders are replaced by their default text and tab stops are dropped,
/// so the cursor ends up after the inserted text. Must run after all other
/// transformers, since those may add snippets.
pub struct PlainTextTransformer;

impl PlainTextTransformer {
    /// Creates a new PlainTextTransformer if the client does not support snippets.
    ///
    /// Returns `None` if the transformation doesn't apply.
    pub(in crate::server::message_handler::completion) fn try_from_env(
        server: &Server,
    ) -> Option<Self> {
        let snippet_support = server
            .client_capabilities
            .as_ref()
            .and_then(|capabilities| capabilities.text_document.as_ref())
            .and_then(|text_document| text_document.completion.as_ref())
            .and_then(|completion| completion.completion_item.as_ref())
            .and_then(|completion_item| completion_item.snippet_support)
            .unwrap_or(false);
        (!snippet_support).then_some(Self)
    }
}

impl CompletionTransformer for PlainTextTransformer {
    fn transform(&self, list: &mut CompletionList) {
        // NOTE: Items may only be snippets through `itemDefaults`.
        list.resolve_insert_text_format();
        for item in list.items.iter_mut() {
            if item.insert_text_format != Some(InsertTextFormat::Snippet) {
                continue;
            }
            if let Some(ref mut insert_text) = item.insert_text {
                *insert_text = strip_snippet(insert_text);
            }
            if let Some(ref mut text_edit) = item.text_edit {
                text_edit.new_text = strip_snippet(&text_edit.new_text);
            }
            item.insert_text_format = Some(InsertTextFormat::PlainText);
        }
    }
}

/// Converts snippet syntax to the text it expands to.
///
/// Handles tab stops (`$1`, `${1}`), placeholders (`${1:foo}`, also nested) and
/// escapes (`\$`, `\}`, `\\`). A `}` outside of a placeholder is literal, which
/// matters for SPARQL group graph patterns.
///
/// `SELECT ${1:*} WHERE {\n\t${0:?s ?p ?o}\n}` -> `SELECT * WHERE {\n\t?s ?p ?o\n}`
fn strip_snippet(snippet: &str) -> String {
    let mut text = String::with_capacity(snippet.len());
    let mut chars = snippet.chars().peekable();
    let mut open_placeholders = 0;
    while let Some(c) = chars.next() {
        match c {
            '\\' if matches!(chars.peek(), Some('$' | '}' | '\\')) => {
                text.extend(chars.next());
            }
            '$' if chars.next_if_eq(&'{').is_some() => {
                while chars.next_if(char::is_ascii_digit).is_some() {}
                chars.next_if_eq(&':');
                open_placeholders += 1;
            }
            '$' if chars.peek().is_some_and(char::is_ascii_digit) => {
                while chars.next_if(char::is_ascii_digit).is_some() {}
            }
            '}' if open_placeholders > 0 => open_placeholders -= 1,
            c => text.push(c),
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::lsp::{
        CompletionItem, CompletionItemKind, ItemDefaults,
        textdocument::{Position, Range, TextEdit},
    };

    fn item(insert_text: &str, format: Option<InsertTextFormat>) -> CompletionItem {
        let mut item = CompletionItem::new(
            "label",
            None,
            None,
            insert_text,
            CompletionItemKind::Snippet,
            None,
        );
        item.insert_text_format = format;
        item
    }

    #[test]
    fn strip_snippet_placeholders() {
        assert_eq!(
            strip_snippet("SELECT ${1:*} WHERE {\n\t${0:?s ?p ?o}\n}"),
            "SELECT * WHERE {\n\t?s ?p ?o\n}"
        );
        assert_eq!(
            strip_snippet("CONSTRUCT {\n  ${1:?s ?p ?o}\n} WHERE {\n  ${0:?s ?p ?o} .\n}"),
            "CONSTRUCT {\n  ?s ?p ?o\n} WHERE {\n  ?s ?p ?o .\n}"
        );
    }

    #[test]
    fn strip_snippet_tab_stops() {
        assert_eq!(strip_snippet("?label .\n  $0"), "?label .\n  ");
        assert_eq!(
            strip_snippet("INSERT DATA {\n  ${0}\n}"),
            "INSERT DATA {\n  \n}"
        );
        assert_eq!(strip_snippet("STR($1)$0"), "STR()");
        assert_eq!(strip_snippet("$10x"), "x");
    }

    #[test]
    fn strip_snippet_nested_placeholders() {
        assert_eq!(strip_snippet("${1:a ${2:b} c}d"), "a b cd");
    }

    #[test]
    fn strip_snippet_escapes() {
        assert_eq!(strip_snippet(r"\$1 ${1:a\}b} \\"), r"$1 a}b \");
        assert_eq!(strip_snippet(r"\n"), r"\n");
        assert_eq!(strip_snippet("costs $"), "costs $");
    }

    #[test]
    fn transform_converts_snippet_items() {
        let mut snippet = item(
            "ASK WHERE {\n  ${0:?s ?p ?o}\n}",
            Some(InsertTextFormat::Snippet),
        );
        snippet.text_edit = Some(TextEdit {
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(0, 0),
            },
            new_text: "?label .\n  $0".to_string(),
        });
        let mut list = CompletionList {
            is_incomplete: false,
            item_defaults: None,
            items: vec![snippet],
        };

        PlainTextTransformer.transform(&mut list);

        let item = &list.items[0];
        assert_eq!(
            item.insert_text.as_deref(),
            Some("ASK WHERE {\n  ?s ?p ?o\n}")
        );
        assert_eq!(item.text_edit.as_ref().unwrap().new_text, "?label .\n  ");
        assert_eq!(item.insert_text_format, Some(InsertTextFormat::PlainText));
    }

    #[test]
    fn transform_keeps_plain_text_items() {
        // `$` is a literal in plain text and must not be stripped.
        let mut list = CompletionList {
            is_incomplete: false,
            item_defaults: None,
            items: vec![
                item("?a$1", Some(InsertTextFormat::PlainText)),
                item("?b$1", None),
            ],
        };

        PlainTextTransformer.transform(&mut list);

        assert_eq!(list.items[0].insert_text.as_deref(), Some("?a$1"));
        assert_eq!(list.items[1].insert_text.as_deref(), Some("?b$1"));
        assert_eq!(list.items[1].insert_text_format, None);
    }

    #[test]
    fn transform_converts_snippets_from_item_defaults() {
        let mut list = CompletionList {
            is_incomplete: false,
            item_defaults: Some(ItemDefaults {
                commit_characters: None,
                edit_range: None,
                insert_text_format: Some(InsertTextFormat::Snippet),
                insert_text_mode: None,
                data: None,
            }),
            items: vec![item("ASK WHERE {\n  ${0:?s ?p ?o}\n}", None)],
        };

        PlainTextTransformer.transform(&mut list);

        assert_eq!(list.item_defaults, None);
        assert_eq!(
            list.items[0].insert_text.as_deref(),
            Some("ASK WHERE {\n  ?s ?p ?o\n}")
        );
        assert_eq!(
            list.items[0].insert_text_format,
            Some(InsertTextFormat::PlainText)
        );
    }
}
