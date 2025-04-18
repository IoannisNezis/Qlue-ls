use super::{
    error::CompletionError,
    utils::{fetch_online_completions, get_prefix_declarations, get_replace_range},
    CompletionContext,
};
use crate::server::{
    lsp::CompletionList, message_handler::completion::context::CompletionLocation, Server,
};
use ll_sparql_parser::ast::{AstNode, QueryUnit};
use tera::Context;
use text_size::TextRange;

pub(super) async fn completions(
    server: &Server,
    context: CompletionContext,
) -> Result<CompletionList, CompletionError> {
    if let CompletionLocation::Object(triple) = &context.location {
        match (context.backend.as_ref(), context.search_term.as_ref()) {
            (Some(backend_name), Some(search_term)) => {
                let prefix_declarations: Vec<_> = get_prefix_declarations(server, &context, triple);
                let range = get_replace_range(&context);
                let query_unit = QueryUnit::cast(context.tree.clone()).ok_or(
                    CompletionError::ResolveError("Could not cast root to QueryUnit".to_string()),
                )?;
                let inject = format!(
                    "{} ?qlue_ls_entity",
                    query_unit.syntax().text().slice(TextRange::new(
                        triple.syntax().text_range().start(),
                        context.anchor_token.unwrap().text_range().end(),
                    ))
                );

                let mut template_context = Context::new();
                template_context.insert("prefixes", &prefix_declarations);
                template_context.insert("context", &inject);
                template_context.insert("search_term", &search_term);

                let items = fetch_online_completions(
                    server,
                    &query_unit,
                    context.backend.as_ref(),
                    &format!("{}-{}", backend_name, "objectCompletion"),
                    template_context,
                    range,
                )
                .await?;
                Ok(CompletionList {
                    is_incomplete: items.len()
                        == server.settings.completion.result_size_limit as usize,
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
