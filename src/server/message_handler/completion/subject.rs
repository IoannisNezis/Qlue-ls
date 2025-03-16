use ll_sparql_parser::syntax_kind::SyntaxKind;

use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::CompletionContext;

pub(super) fn completions(context: CompletionContext) -> Vec<CompletionItem> {
    let mut res = Vec::new();
    if context
        .continuations
        .contains(&SyntaxKind::GroupGraphPatternSub)
        || context.continuations.contains(&SyntaxKind::TriplesBlock)
    {
        res.push(CompletionItem::new(
            "subject filler",
            "Hier könnte ihre subject completion stehen",
            "<subject> ",
            CompletionItemKind::Value,
            InsertTextFormat::PlainText,
        ));
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
                "Filter the results",
                "FILTER ( $0 )",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
            CompletionItem::new(
                "BIND",
                "Bind a new variable",
                "BIND ($1 AS ?$0)",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
            CompletionItem::new(
                "VALUES",
                "Inline data definition",
                "VALUES ?$1 { $0 }",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
            CompletionItem::new(
                "SERVICE",
                "Collect data from a fedarated SPARQL endpoint",
                "SERVICE <$1> {\n  $0\n}",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
            CompletionItem::new(
                "MINUS",
                "Subtract data",
                "MINUS { $0 }",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
            CompletionItem::new(
                "OPTIONAL",
                "Optional graphpattern",
                "OPTIONAL { $0 }",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
            CompletionItem::new(
                "UNION",
                "Union of two results",
                "{\n  $1\n}\nUNION\n{\n  $0\n}",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
            CompletionItem::new(
                "Sub select",
                "Sub select query",
                "{\n  Select * WHERE {\n    $0\n  }\n}",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
            ),
        ]);
    }
    res
}
