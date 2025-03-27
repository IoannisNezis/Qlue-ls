use super::{utils::fetch_online_completions, CompletionContext};
use crate::server::{
    lsp::{CompletionItem, CompletionItemKind, InsertTextFormat},
    Server,
};
use ll_sparql_parser::{
    ast::{AstNode, QueryUnit},
    syntax_kind::SyntaxKind,
};
use tera::Context;

pub(super) async fn completions(
    server: &Server,
    context: CompletionContext,
) -> Vec<CompletionItem> {
    let mut res = Vec::new();
    if context
        .continuations
        .contains(&SyntaxKind::GroupGraphPatternSub)
        || context.continuations.contains(&SyntaxKind::TriplesBlock)
        || context.continuations.contains(&SyntaxKind::DataBlockValue)
    {
        if let Some(search_term) = context.search_term {
            let mut template_context = Context::new();
            template_context.insert("search_term", &search_term);
            let query_unit = QueryUnit::cast(context.tree).unwrap();
            match fetch_online_completions(
                server,
                &query_unit,
                "subject_completion.rq",
                template_context,
            )
            .await
            {
                Ok(online_completions) => res.extend(online_completions),
                Err(err) => {
                    log::error!("{:?}", err);
                }
            };
        }
    }
    if context
        .continuations
        .contains(&SyntaxKind::GroupGraphPatternSub)
        || context
            .continuations
            .contains(&SyntaxKind::GraphPatternNotTriples)
    {
        res.append(&mut vec![
            CompletionItem::new(
                "FILTER",
                Some("Filter the results".to_string()),
                None,
                "FILTER ( $0 )",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "BIND",
                Some("Bind a new variable".to_string()),
                None,
                "BIND ($1 AS ?$0)",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "VALUES",
                Some("Inline data definition".to_string()),
                None,
                "VALUES ?$1 { $0 }",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "SERVICE",
                Some("Collect data from a fedarated SPARQL endpoint".to_string()),
                None,
                "SERVICE <$1> {\n  $0\n}",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "MINUS",
                Some("Subtract data".to_string()),
                None,
                "MINUS { $0 }",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "OPTIONAL",
                Some("Optional graphpattern".to_string()),
                None,
                "OPTIONAL { $0 }",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "UNION",
                Some("Union of two results".to_string()),
                None,
                "{\n  $1\n}\nUNION\n{\n  $0\n}",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "Sub select",
                Some("Sub select query".to_string()),
                None,
                "{\n  Select * WHERE {\n    $0\n  }\n}",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
        ]);
    }
    res
}
