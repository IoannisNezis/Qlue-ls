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

static QUERY_TEMPLATE: &str = "predicate_completion.rq";

pub(super) async fn completions(
    server: &Server,
    context: CompletionContext,
) -> Vec<CompletionItem> {
    if let CompletionLocation::Predicate(triple) = &context.location {
        let range = get_replace_range(&context);
        let query_unit = QueryUnit::cast(context.tree).unwrap();
        let mut template_context = Context::new();
        let inject = query_unit.syntax().text().slice(TextRange::new(
            triple.syntax().text_range().start(),
            context.anchor_token.unwrap().text_range().end(),
        ));
        template_context.insert("context", &inject.to_string());
        template_context.insert(
            "prefixes",
            &triple
                .used_prefixes()
                .iter()
                .filter_map(|prefix| server.tools.uri_converter.find_by_prefix(prefix).ok())
                .map(|record| (record.prefix.clone(), record.uri_prefix.clone()))
                .collect::<Vec<_>>(),
        );

        match fetch_online_completions(server, &query_unit, QUERY_TEMPLATE, template_context, range)
            .await
        {
            Ok(online_completions) => online_completions,
            Err(err) => {
                log::error!("{:?}", err);
                vec![]
            }
        }
    } else {
        panic!("object completions requested for non object location");
    }
}
