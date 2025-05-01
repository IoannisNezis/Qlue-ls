use super::{
    error::CompletionError,
    utils::{
        fetch_online_completions, get_prefix_declarations, get_replace_range, reduce_path,
        to_completion_items,
    },
    CompletionContext,
};
use crate::server::{
    lsp::CompletionList, message_handler::completion::context::CompletionLocation, Server,
};
use futures::lock::Mutex;
use ll_sparql_parser::{
    ast::{AstNode, QueryUnit, Triple},
    syntax_kind::SyntaxKind,
};
use std::{collections::HashSet, rc::Rc};
use tera::Context;
use text_size::TextSize;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    context: CompletionContext,
) -> Result<CompletionList, CompletionError> {
    if let CompletionLocation::Predicate(triple) = &context.location {
        match (context.backend.as_ref(), context.search_term.as_ref()) {
            (Some(backend_name), Some(search_term)) => {
                let range = get_replace_range(&context);
                let mut template_context = Context::new();
                let query_unit = QueryUnit::cast(context.tree.clone()).ok_or(
                    CompletionError::ResolveError("Could not cast root to QueryUnit".to_string()),
                )?;
                let prefixes = get_prefix_declarations(
                    &*server_rc.lock().await,
                    &context,
                    triple.used_prefixes(),
                );
                let inject = compute_inject_context(
                    triple,
                    context.anchor_token.unwrap().text_range().end(),
                    context.continuations,
                )
                .ok_or(CompletionError::ResolveError(
                    "Could not build inject-string for the template".to_string(),
                ))?;
                template_context.insert("context", &inject);
                template_context.insert("prefixes", &prefixes);
                template_context.insert("search_term", search_term);
                let items = to_completion_items(
                    fetch_online_completions(
                        server_rc.clone(),
                        &query_unit,
                        context.backend.as_ref(),
                        &format!("{}-{}", backend_name, "predicateCompletion"),
                        template_context,
                    )
                    .await?,
                    range,
                    Some("triggerNewCompletion"),
                );

                Ok(CompletionList {
                    is_incomplete: items.len()
                        == server_rc.lock().await.settings.completion.result_size_limit as usize,
                    item_defaults: None,
                    items,
                })
            }
            _ => {
                log::info!("No Backend or search term");
                Ok(CompletionList {
                    is_incomplete: false,
                    item_defaults: None,
                    items: vec![],
                })
            }
        }
    } else {
        panic!("object completions requested for non object location");
    }
}

fn compute_inject_context(
    triple: &Triple,
    offset: TextSize,
    continuations: HashSet<SyntaxKind>,
) -> Option<String> {
    let subject_string = triple.subject()?.text();
    if continuations.contains(&SyntaxKind::PropertyListPath)
        || continuations.contains(&SyntaxKind::PropertyListPathNotEmpty)
    {
        Some(format!("{} ?qlue_ls_entity []", subject_string))
    } else {
        let properties = triple.properties_list_path()?.properties();
        if continuations.contains(&SyntaxKind::VerbPath) {
            Some(format!("{} ?qlue_ls_entity []", triple.text()))
        } else if properties.len() == 1 {
            reduce_path(&subject_string, &properties[0].verb, "[]", offset)
        } else {
            let (last_prop, prev_prop) = properties.split_last()?;
            Some(format!(
                "{} {} . {}",
                subject_string,
                prev_prop
                    .iter()
                    .map(|prop| prop.text())
                    .collect::<Vec<_>>()
                    .join(" ; "),
                reduce_path(&subject_string, &last_prop.verb, "[]", offset)?
            ))
        }
    }
}
