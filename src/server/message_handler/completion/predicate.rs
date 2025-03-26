use ll_sparql_parser::ast::{AstNode, QueryUnit};
use tera::Context;
use text_size::TextRange;

use crate::server::{
    fetch::fetch_sparql_result,
    lsp::{CompletionItem, CompletionItemKind, InsertTextFormat},
    message_handler::completion::context::CompletionLocation,
    Server,
};

use super::{utils::compress_rdf_term, CompletionContext};

pub(super) async fn completions(
    server: &Server,
    context: CompletionContext,
) -> Vec<CompletionItem> {
    if let CompletionLocation::Predicate(triple) = context.location {
        if let Some(backend) = &server.state.backend {
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
            let query = server
                .tools
                .tera
                .render("predicate_completion.rq", &template_context)
                .expect("Template should render");

            match fetch_sparql_result(&backend.url, &query).await {
                Ok(result) => result
                    .results
                    .bindings
                    .into_iter()
                    .map(|binding| {
                        let pred = binding.get("pred").unwrap();
                        let count = binding.get("count").unwrap();
                        let (value, import_edit) = compress_rdf_term(server, &query_unit, pred);
                        CompletionItem::new(
                            &value,
                            &format!("{} occurences", count),
                            &format!("{} ", value),
                            CompletionItemKind::Value,
                            InsertTextFormat::PlainText,
                            import_edit.map(|edit| vec![edit]),
                        )
                    })
                    .collect(),
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
