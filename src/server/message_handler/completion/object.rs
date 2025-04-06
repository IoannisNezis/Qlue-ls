use super::{
    error::CompletionError,
    utils::{fetch_online_completions, get_prefix_declarations, get_replace_range},
    CompletionContext,
};
use crate::server::{
    lsp::CompletionItem, message_handler::completion::context::CompletionLocation, Server,
};
use ll_sparql_parser::ast::{AstNode, QueryUnit};
use tera::Context;
use text_size::TextRange;

static QUERY_TEMPLATE: &str = "object_completion.rq";

pub(super) async fn completions(
    server: &Server,
    context: CompletionContext,
) -> Result<Vec<CompletionItem>, CompletionError> {
    if let CompletionLocation::Object(triple) = &context.location {
        let search_term = context
            .search_term
            .as_ref()
            .ok_or(CompletionError::ResolveError(
                "Could not detect search term for object completion".to_string(),
            ))?;
        let prefix_declarations: Vec<_> = get_prefix_declarations(server, &context, triple);
        let range = get_replace_range(&context);
        let query_unit = QueryUnit::cast(context.tree.clone()).ok_or(
            CompletionError::ResolveError("Could not cast root to QueryUnit".to_string()),
        )?;
        let inject = query_unit
            .syntax()
            .text()
            .slice(TextRange::new(
                triple.syntax().text_range().start(),
                context.anchor_token.unwrap().text_range().end(),
            ))
            .to_string();
        let mut template_context = Context::new();
        template_context.insert("prefixes", &prefix_declarations);
        template_context.insert("context", &inject);
        template_context.insert("search_term", &search_term);

        fetch_online_completions(
            server,
            &query_unit,
            context.backend.as_ref(),
            QUERY_TEMPLATE,
            template_context,
            range,
        )
        .await
    } else {
        panic!("object completions requested for non object location");
    }
}
