use std::{collections::HashSet, rc::Rc};

use super::{
    context::{CompletionContext, CompletionLocation},
    error::CompletionError,
    utils::{get_replace_range, to_completion_items},
};
use crate::server::{
    lsp::CompletionList,
    message_handler::completion::utils::{
        fetch_online_completions, get_prefix_declarations, reduce_path,
    },
    Server,
};
use futures::lock::Mutex;
use ll_sparql_parser::{
    ast::{AstNode, PropertyListPath, QueryUnit},
    syntax_kind::SyntaxKind,
};
use tera::Context;
use text_size::TextSize;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    context: CompletionContext,
) -> Result<CompletionList, CompletionError> {
    if let CompletionLocation::BlankNodeProperty(blank_node_props) = &context.location {
        match (
            context.backend.as_ref(),
            context.search_term.as_ref(),
            context.anchor_token.as_ref(),
        ) {
            (Some(backend_name), Some(search_term), Some(anchor_token)) => {
                let query_unit = QueryUnit::cast(context.tree.clone()).ok_or(
                    CompletionError::ResolveError("Could not cast root to QueryUnit".to_string()),
                )?;
                let range = get_replace_range(&context);
                let prefixes = get_prefix_declarations(
                    &*server_rc.lock().await,
                    &context,
                    blank_node_props.used_prefixes(),
                );
                let inject = compute_inject_context(
                    blank_node_props.property_list(),
                    anchor_token.text_range().end(),
                    context.continuations,
                )
                .ok_or(CompletionError::ResolveError(
                    "Could not build inject-string for the template".to_string(),
                ))?;

                let mut template_context = Context::new();
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
    props: Option<PropertyListPath>,
    offset: TextSize,
    continuations: HashSet<SyntaxKind>,
) -> Option<String> {
    if continuations.contains(&SyntaxKind::PropertyListPath)
        || continuations.contains(&SyntaxKind::PropertyListPathNotEmpty)
        || props.is_none()
    {
        Some(format!("[] ?qlue_ls_entity []"))
    } else {
        let properties = props.unwrap().properties();
        if continuations.contains(&SyntaxKind::VerbPath) {
            Some(format!("[] ?qlue_ls_entity []"))
        } else if properties.len() == 1 {
            reduce_path("[]", &properties[0].verb, "[]", offset)
        } else {
            let (last_prop, prev_prop) = properties.split_last()?;
            Some(format!(
                "[] {} . {}",
                prev_prop
                    .iter()
                    .map(|prop| prop.text())
                    .collect::<Vec<_>>()
                    .join(" ; "),
                reduce_path("[]", &last_prop.verb, "[]", offset)?
            ))
        }
    }
}
