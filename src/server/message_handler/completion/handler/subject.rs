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
        InsertTextFormat,
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
        static_completions()
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

fn static_completions() -> Vec<CompletionItem> {
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
            .insert_text("FILTER ($0)")
            .insert_text_format(InsertTextFormat::Snippet)
            .command(trigger_completion_command.clone())
            .build(),
        CompletionItemBuilder::new()
            .label("BIND")
            .kind(CompletionItemKind::Snippet)
            .detail("Bind a new variable")
            .sort_text("00002")
            .insert_text("BIND ($1 AS ?$0)")
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("VALUES")
            .kind(CompletionItemKind::Snippet)
            .detail("Inline data definition")
            .sort_text("00003")
            .insert_text("VALUES ?$1 { $0 }")
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("SERVICE")
            .kind(CompletionItemKind::Snippet)
            .detail("Collect data from a fedarated SPARQL endpoint")
            .sort_text("00004")
            .insert_text("SERVICE $1 {\n  $0\n}")
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("MINUS")
            .kind(CompletionItemKind::Snippet)
            .detail("Subtract data")
            .sort_text("00005")
            .insert_text("MINUS { $0 }")
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("OPTIONAL")
            .kind(CompletionItemKind::Snippet)
            .detail("Optional graphpattern")
            .sort_text("00006")
            .insert_text("OPTIONAL { $0 }")
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("UNION")
            .kind(CompletionItemKind::Snippet)
            .detail("Union of two results")
            .sort_text("00007")
            .insert_text("{\n  $1\n}\nUNION\n{\n  $0\n}")
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
        CompletionItemBuilder::new()
            .label("Sub select")
            .kind(CompletionItemKind::Snippet)
            .detail("Sub select query")
            .sort_text("00008")
            .insert_text("{\n  SELECT * WHERE {\n    $0\n  }\n}")
            .insert_text_format(InsertTextFormat::Snippet)
            .build(),
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
