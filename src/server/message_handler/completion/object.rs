use super::{
    utils::{fetch_online_completions, get_replace_range},
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
) -> Vec<CompletionItem> {
    if let CompletionLocation::Object(triple) = &context.location {
        if let Some(search_term) = context.search_term.as_ref() {
            log::info!("canary1");
            let prefix_declarations: Vec<_> = triple
                .used_prefixes()
                .into_iter()
                .filter_map(|prefix| server.tools.uri_converter.find_by_prefix(&prefix).ok())
                .map(|record| (&record.prefix, &record.uri_prefix))
                .collect();
            let range = get_replace_range(&context);
            let query_unit = QueryUnit::cast(context.tree).unwrap();
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

            log::info!("canary2");
            match fetch_online_completions(
                server,
                &query_unit,
                QUERY_TEMPLATE,
                template_context,
                range,
            )
            .await
            {
                Ok(res) => res,
                Err(err) => {
                    log::error!("{:?}", err);
                    vec![]
                }
            }
        } else {
            vec![]
        }
    } else {
        panic!("object completions requested for non object location");
    }
}
