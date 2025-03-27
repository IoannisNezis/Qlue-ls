use ll_sparql_parser::ast::{AstNode, QueryUnit};
use tera::Context;
use text_size::TextRange;

use crate::server::{
    fetch::fetch_sparql_result,
    lsp::{CompletionItem, CompletionItemKind, InsertTextFormat},
    message_handler::completion::context::CompletionLocation,
    Server,
};

use super::{
    utils::{compress_rdf_term, fetch_online_completions},
    CompletionContext,
};

pub(super) async fn completions(
    server: &Server,
    context: CompletionContext,
) -> Vec<CompletionItem> {
    if let CompletionLocation::Object(triple) = context.location {
        let prefix_declarations: Vec<_> = triple
            .used_prefixes()
            .into_iter()
            .filter_map(|prefix| server.tools.uri_converter.find_by_prefix(&prefix).ok())
            .map(|record| (&record.prefix, &record.uri_prefix))
            .collect();
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
        match fetch_online_completions(
            server,
            &query_unit,
            "object_completion.rq",
            template_context,
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
        panic!("object completions requested for non object location");
    }
}
