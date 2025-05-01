use std::rc::Rc;

use super::{
    error::CompletionError,
    utils::{fetch_online_completions, get_replace_range, to_completion_items},
    variable, CompletionContext,
};
use crate::server::{
    lsp::{Command, CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat},
    Server,
};
use futures::lock::Mutex;
use ll_sparql_parser::{
    ast::{AstNode, QueryUnit},
    syntax_kind::SyntaxKind,
};
use tera::Context;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    context: CompletionContext,
) -> Result<CompletionList, CompletionError> {
    let mut items = Vec::new();
    let mut is_incomplete = false;
    if [
        SyntaxKind::GroupGraphPatternSub,
        SyntaxKind::TriplesBlock,
        SyntaxKind::DataBlockValue,
        SyntaxKind::GraphNodePath,
    ]
    .iter()
    .any(|kind| context.continuations.contains(kind))
    {
        match (context.backend.as_ref(), context.search_term.as_ref()) {
            (Some(backend_name), Some(search_term)) => {
                let mut template_context = Context::new();
                template_context.insert("search_term", search_term);
                template_context.insert::<Vec<(&str, &str)>, &str>("prefixes", &vec![]);
                let range = get_replace_range(&context);
                let query_unit = QueryUnit::cast(context.tree.clone()).unwrap();

                match fetch_online_completions(
                    server_rc.clone(),
                    &query_unit,
                    context.backend.as_ref(),
                    &format!("{}-{}", backend_name, "subjectCompletion"),
                    template_context,
                )
                .await
                {
                    Ok(online_completions) => {
                        is_incomplete = online_completions.len()
                            == server_rc.lock().await.settings.completion.result_size_limit
                                as usize;
                        items.extend(to_completion_items(
                            online_completions,
                            range,
                            Some("triggerNewCompletion"),
                        ));
                    }
                    Err(err) => {
                        log::error!("{:?}", err);
                    }
                };
            }
            _ => log::info!("No Backend or search term"),
        }
    }
    if context
        .continuations
        .contains(&SyntaxKind::GroupGraphPatternSub)
        || context
            .continuations
            .contains(&SyntaxKind::GraphPatternNotTriples)
    {
        items.append(&mut vec![
            CompletionItem {
                label: "FILTER".to_string(),
                label_details: None,
                kind: CompletionItemKind::Snippet,
                detail: Some("Filter the results".to_string()),
                sort_text: None,
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
                sort_text: None,
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
                sort_text: None,
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
                sort_text: None,
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
                sort_text: None,
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
                sort_text: None,
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
                sort_text: None,
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
                sort_text: None,
                insert_text: Some("{\n  SELECT * WHERE {\n    $0\n  }\n}".to_string()),
                text_edit: None,
                insert_text_format: Some(InsertTextFormat::Snippet),
                additional_text_edits: None,
            },
        ]);
    }

    items.extend(variable::completions_transformed(context)?.items);
    Ok(CompletionList {
        is_incomplete,
        item_defaults: None,
        items,
    })
}
