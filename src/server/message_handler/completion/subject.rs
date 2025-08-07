use std::rc::Rc;

use super::{
    error::CompletionError,
    utils::{dispatch_completion_query, CompletionTemplate},
    variable, CompletionEnvironment,
};
use crate::server::{
    lsp::{Command, CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat},
    Server,
};
use futures::lock::Mutex;
use ll_sparql_parser::syntax_kind::SyntaxKind;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let mut items = (environment
        .continuations
        .contains(&SyntaxKind::GroupGraphPatternSub)
        || environment
            .continuations
            .contains(&SyntaxKind::GraphPatternNotTriples))
    .then_some(static_completions())
    .unwrap_or_default();
    let mut is_incomplete = false;

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
        if let Ok(online_completions) = dispatch_completion_query(
            server_rc.clone(),
            &environment,
            template_context,
            CompletionTemplate::SubjectCompletion,
            true,
        )
        .await
        {
            items.extend(online_completions.items);
            is_incomplete = online_completions.is_incomplete;
        }
    }
    items.extend(
        variable::completions_transformed(server_rc, &environment)
            .await?
            .items,
    );
    Ok(CompletionList {
        is_incomplete,
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
            sort_text: Some("00008".to_string()),
            filter_text: None,
            insert_text: Some("{\n  SELECT * WHERE {\n    $0\n  }\n}".to_string()),
            text_edit: None,
            insert_text_format: Some(InsertTextFormat::Snippet),
            additional_text_edits: None,
        },
    ]
}
