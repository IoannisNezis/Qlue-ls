use std::rc::Rc;

use super::super::{
    CompletionEnvironment,
    error::CompletionError,
    utils::{CompletionTemplate, dispatch_completion_query, matches_search_term},
};
use crate::server::{
    Server,
    lsp::{
        Command, CompletionItem, CompletionItemBuilder, CompletionItemKind, CompletionList,
        InsertTextFormat, textdocument::Range, textdocument::TextEdit,
    },
};
use futures::lock::Mutex;
use ll_sparql_parser::syntax_kind::SyntaxKind;

pub async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: &CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let mut items: Vec<CompletionItem> = if environment
        .continuations
        .contains(&SyntaxKind::GroupGraphPatternSub)
        || environment
            .continuations
            .contains(&SyntaxKind::GraphPatternNotTriples)
    {
        static_completions(&environment.replace_range)
            .into_iter()
            .filter(|item| matches_search_term(&item.label, environment.search_term.as_deref()))
            .collect()
    } else {
        Vec::default()
    };

    // NOTE: entity subject completions are only triggered if the search term is atleast N long.
    let trigger_threshold = server_rc
        .lock()
        .await
        .settings
        .completion
        .subject_completion_trigger_length;

    if environment
        .search_term
        .as_ref()
        .is_some_and(|search_term| search_term.len() >= trigger_threshold as usize)
        && [
            SyntaxKind::GroupGraphPatternSub,
            SyntaxKind::TriplesBlock,
            SyntaxKind::DataBlockValue,
            SyntaxKind::GraphNodePath,
        ]
        .iter()
        .any(|kind| environment.continuations.contains(kind))
    {
        let template_context = environment.template_context().await;
        match dispatch_completion_query(
            server_rc.clone(),
            environment,
            template_context,
            CompletionTemplate::SubjectCompletion,
            true,
        )
        .await
        {
            Ok(online_completions) => {
                items.extend(online_completions.items);
            }
            Err(err) => {
                tracing::error!("Completion query failed: {err:?}");
            }
        }
    }

    Ok(CompletionList {
        is_incomplete: true,
        item_defaults: None,
        items,
    })
}

/// `replace_range` covers the whole search term. Without it a client falls back
/// to its own word scan, which stops at the space in a multi word label -- so
/// accepting "Sub select" after typing "sub s" left the "sub " behind.
fn static_completions(replace_range: &Range) -> Vec<CompletionItem> {
    let trigger_completion_command = Command {
        title: "triggerNewCompletion".to_string(),
        command: "triggerNewCompletion".to_string(),
        arguments: None,
    };
    vec![
        CompletionItemBuilder::new()
            .label("FILTER")
            .kind(CompletionItemKind::Snippet)
            .detail("Filter the results")
            .sort_text("00001")
            .text_edit(TextEdit::new(replace_range.clone(), "FILTER ($0)"))
            .insert_text_format(InsertTextFormat::Snippet)
            .command(trigger_completion_command.clone())
            .build(),
        CompletionItemBuilder::new()
            .label("BIND")
            .kind(CompletionItemKind::Snippet)
            .detail("Bind a new variable")
            .sort_text("00002")
            .text_edit(TextEdit::new(replace_range.clone(), "BIND ($1 AS ?$0)"))
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("VALUES")
            .kind(CompletionItemKind::Snippet)
            .detail("Inline data definition")
            .sort_text("00003")
            .text_edit(TextEdit::new(replace_range.clone(), "VALUES ?$1 { $0 }"))
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("SERVICE")
            .kind(CompletionItemKind::Snippet)
            .detail("Collect data from a fedarated SPARQL endpoint")
            .sort_text("00004")
            .text_edit(TextEdit::new(
                replace_range.clone(),
                "SERVICE $1 {\n  $0\n}",
            ))
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("MINUS")
            .kind(CompletionItemKind::Snippet)
            .detail("Subtract data")
            .sort_text("00005")
            .text_edit(TextEdit::new(replace_range.clone(), "MINUS { $0 }"))
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("OPTIONAL")
            .kind(CompletionItemKind::Snippet)
            .detail("Optional graphpattern")
            .sort_text("00006")
            .text_edit(TextEdit::new(replace_range.clone(), "OPTIONAL { $0 }"))
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("UNION")
            .kind(CompletionItemKind::Snippet)
            .detail("Union of two results")
            .sort_text("00007")
            .text_edit(TextEdit::new(
                replace_range.clone(),
                "{\n  $1\n}\nUNION\n{\n  $0\n}",
            ))
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("Sub select")
            .kind(CompletionItemKind::Snippet)
            .detail("Sub select query")
            .sort_text("00008")
            .text_edit(TextEdit::new(
                replace_range.clone(),
                "{\n  SELECT * WHERE {\n    $0\n  }\n}",
            ))
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
    ]
}

#[cfg(test)]
mod tests {
    use super::{matches_search_term, static_completions};
    use crate::server::lsp::textdocument::Range;

    fn filter_completions(search_term: Option<&str>) -> Vec<String> {
        let replace_range = Range::new(0, 0, 0, 0);
        static_completions(&replace_range)
            .into_iter()
            .filter(|item| matches_search_term(&item.label, search_term))
            .map(|item| item.label)
            .collect()
    }

    #[test]
    fn no_search_term_returns_all_keywords() {
        let labels = filter_completions(None);
        assert_eq!(labels.len(), 8);
        assert!(labels.contains(&"FILTER".to_string()));
        assert!(labels.contains(&"BIND".to_string()));
        assert!(labels.contains(&"VALUES".to_string()));
        assert!(labels.contains(&"SERVICE".to_string()));
        assert!(labels.contains(&"MINUS".to_string()));
        assert!(labels.contains(&"OPTIONAL".to_string()));
        assert!(labels.contains(&"UNION".to_string()));
        assert!(labels.contains(&"Sub select".to_string()));
    }

    #[test]
    fn every_completion_replaces_the_search_term() {
        // NOTE: without a range the client falls back to a word scan, which
        // stops at the space in "sub s" and leaves the "sub " behind.
        let replace_range = Range::new(1, 2, 1, 7);
        for item in static_completions(&replace_range) {
            let text_edit = item
                .text_edit
                .unwrap_or_else(|| panic!("{} has no text edit", item.label));
            assert_eq!(text_edit.range, replace_range, "{}", item.label);
        }
    }

    #[test]
    fn filter_prefix_returns_filter() {
        let labels = filter_completions(Some("FI"));
        assert_eq!(labels, vec!["FILTER"]);
    }

    #[test]
    fn filter_prefix_case_insensitive() {
        let labels = filter_completions(Some("fi"));
        assert_eq!(labels, vec!["FILTER"]);
    }

    #[test]
    fn bind_prefix_returns_bind() {
        let labels = filter_completions(Some("BI"));
        assert_eq!(labels, vec!["BIND"]);
    }

    #[test]
    fn optional_prefix_returns_optional() {
        let labels = filter_completions(Some("OP"));
        assert_eq!(labels, vec!["OPTIONAL"]);
    }

    #[test]
    fn service_and_sub_select_share_prefix() {
        let labels = filter_completions(Some("S"));
        assert_eq!(labels.len(), 2);
        assert!(labels.contains(&"SERVICE".to_string()));
        assert!(labels.contains(&"Sub select".to_string()));
    }

    #[test]
    fn non_keyword_prefix_returns_empty() {
        let labels = filter_completions(Some("Germany"));
        assert!(labels.is_empty());
    }

    #[test]
    fn random_text_returns_empty() {
        let labels = filter_completions(Some("xyz"));
        assert!(labels.is_empty());
    }

    #[test]
    fn partial_match_not_prefix_returns_empty() {
        // "ILTER" is part of "FILTER" but not a prefix
        let labels = filter_completions(Some("ILTER"));
        assert!(labels.is_empty());
    }
}
