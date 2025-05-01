use std::rc::Rc;

use super::{
    context::{CompletionContext, CompletionLocation},
    error::CompletionError,
    utils::{get_replace_range, to_completion_items},
};
use crate::server::{
    lsp::CompletionList,
    message_handler::completion::utils::{fetch_online_completions, get_prefix_declarations},
    Server,
};
use futures::lock::Mutex;
use ll_sparql_parser::ast::{AstNode, QueryUnit};
use tera::Context;
use text_size::TextRange;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    context: CompletionContext,
) -> Result<CompletionList, CompletionError> {
    if let CompletionLocation::BlankNodeObject(blank_node_props) = &context.location {
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

                let inject = format!(
                    "[] {} ?qlue_ls_entity",
                    query_unit.syntax().text().slice(TextRange::new(
                        blank_node_props
                            .property_list()
                            .expect("If there is a object there should be a propertyList")
                            .syntax()
                            .text_range()
                            .start(),
                        anchor_token.text_range().end(),
                    ))
                );

                let mut template_context = Context::new();
                template_context.insert("context", &inject);
                template_context.insert("prefixes", &prefixes);
                template_context.insert("search_term", search_term);
                let items = to_completion_items(
                    fetch_online_completions(
                        server_rc.clone(),
                        &query_unit,
                        context.backend.as_ref(),
                        &format!("{}-{}", backend_name, "objectCompletion"),
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
