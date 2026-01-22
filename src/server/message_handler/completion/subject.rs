use std::rc::Rc;

use super::{
    CompletionEnvironment,
    error::CompletionError,
    utils::{CompletionTemplate, dispatch_completion_query, matches_search_term},
};
use crate::server::{
    Server,
    lsp::{Command, CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat},
};
use futures::lock::Mutex;
use ll_sparql_parser::syntax_kind::SyntaxKind;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: &CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let mut items: Vec<CompletionItem> = (environment
        .continuations
        .contains(&SyntaxKind::GroupGraphPatternSub)
        || environment
            .continuations
            .contains(&SyntaxKind::GraphPatternNotTriples))
    .then_some(
        static_completions()
            .into_iter()
            .filter(|item| matches_search_term(&item.label, environment.search_term.as_deref()))
            .collect(),
    )
    .unwrap_or_default();

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
        .is_some_and(|search_term| search_term.len() > trigger_threshold as usize)
    {
        if [
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
                &environment,
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
                    log::error!("Completion query failed: {err:?}");
                }
            }
        }
    }

    Ok(CompletionList {
        is_incomplete: true,
        item_defaults: None,
        items,
    })
}

fn static_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "FILTER".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Filter the results".to_string()),
            documentation: None,
            sort_text: Some("00001".to_string()),
            filter_text: None,
            insert_text: Some("FILTER ($0)".to_string()),
            text_edit: None,
            insert_text_format: Some(InsertTextFormat::Snippet),
            additional_text_edits: None,
            command: Some(Command {
                title: "triggerNewCompletion".to_string(),
                command: "triggerNewCompletion".to_string(),
                arguments: None,
            }),
        },
        CompletionItem {
            command: None,
            label: "BIND".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Bind a new variable".to_string()),
            documentation: None,
            sort_text: Some("00002".to_string()),
            filter_text: None,
            insert_text: Some("BIND ($1 AS ?$0)".to_string()),
            text_edit: None,
            insert_text_format: Some(InsertTextFormat::Snippet),
            additional_text_edits: None,
        },
        CompletionItem {
            command: None,
            label: "VALUES".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Inline data definition".to_string()),
            documentation: None,
            sort_text: Some("00003".to_string()),
            filter_text: None,
            insert_text: Some("VALUES ?$1 { $0 }".to_string()),
            text_edit: None,
            insert_text_format: Some(InsertTextFormat::Snippet),
            additional_text_edits: None,
        },
        CompletionItem {
            command: None,
            label: "SERVICE".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Collect data from a fedarated SPARQL endpoint".to_string()),
            documentation: None,
            sort_text: Some("00004".to_string()),
            filter_text: None,
            insert_text: Some("SERVICE $1 {\n  $0\n}".to_string()),
            text_edit: None,
            insert_text_format: Some(InsertTextFormat::Snippet),
            additional_text_edits: None,
        },
        CompletionItem {
            command: None,
            label: "MINUS".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Subtract data".to_string()),
            documentation: None,
            sort_text: Some("00005".to_string()),
            filter_text: None,
            insert_text: Some("MINUS { $0 }".to_string()),
            text_edit: None,
            insert_text_format: Some(InsertTextFormat::Snippet),
            additional_text_edits: None,
        },
        CompletionItem {
            command: None,
            label: "OPTIONAL".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Optional graphpattern".to_string()),
            documentation: None,
            sort_text: Some("00006".to_string()),
            filter_text: None,
            insert_text: Some("OPTIONAL { $0 }".to_string()),
            text_edit: None,
            insert_text_format: Some(InsertTextFormat::Snippet),
            additional_text_edits: None,
        },
        CompletionItem {
            command: None,
            label: "UNION".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Union of two results".to_string()),
            documentation: None,
            sort_text: Some("00007".to_string()),
            filter_text: None,
            insert_text: Some("{\n  $1\n}\nUNION\n{\n  $0\n}".to_string()),
            text_edit: None,
            insert_text_format: Some(InsertTextFormat::Snippet),
            additional_text_edits: None,
        },
        CompletionItem {
            command: None,
            label: "Sub select".to_string(),
            label_details: None,
            kind: CompletionItemKind::Snippet,
            detail: Some("Sub select query".to_string()),
            documentation: None,
            sort_text: Some("00008".to_string()),
            filter_text: None,
            insert_text: Some("{\n  SELECT * WHERE {\n    $0\n  }\n}".to_string()),
            text_edit: None,
            insert_text_format: Some(InsertTextFormat::Snippet),
            additional_text_edits: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{matches_search_term, static_completions};

    fn filter_completions(search_term: Option<&str>) -> Vec<String> {
        static_completions()
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
