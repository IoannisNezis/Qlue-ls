use ll_sparql_parser::syntax_kind::SyntaxKind;

use crate::server::lsp::{CompletionItem, CompletionItemKind, InsertTextFormat};

use super::CompletionContext;

//     let query = r#"PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
// PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
// SELECT ?type ?label ?count WHERE {
//   {
//     SELECT ?type (COUNT(?s) AS ?count) WHERE {
//       ?s rdf:type ?type .
//     }
//     GROUP BY ?type
//   }
//   ?type rdfs:label ?label .
// }
// ORDER BY DESC(?count)
// LIMIT 20
// "#;

pub(super) async fn completions(context: CompletionContext) -> Vec<CompletionItem> {
    let mut res = Vec::new();
    if context
        .continuations
        .contains(&SyntaxKind::GroupGraphPatternSub)
        || context.continuations.contains(&SyntaxKind::TriplesBlock)
        || context.continuations.contains(&SyntaxKind::DataBlockValue)
    {
        res.push(CompletionItem::new(
            "subject filler",
            "Hier könnte ihre subject completion stehen",
            "<subject> ",
            CompletionItemKind::Value,
            InsertTextFormat::PlainText,
            None,
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
                None,
            ),
            CompletionItem::new(
                "BIND",
                "Bind a new variable",
                "BIND ($1 AS ?$0)",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "VALUES",
                "Inline data definition",
                "VALUES ?$1 { $0 }",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "SERVICE",
                "Collect data from a fedarated SPARQL endpoint",
                "SERVICE <$1> {\n  $0\n}",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "MINUS",
                "Subtract data",
                "MINUS { $0 }",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "OPTIONAL",
                "Optional graphpattern",
                "OPTIONAL { $0 }",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "UNION",
                "Union of two results",
                "{\n  $1\n}\nUNION\n{\n  $0\n}",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
            CompletionItem::new(
                "Sub select",
                "Sub select query",
                "{\n  Select * WHERE {\n    $0\n  }\n}",
                CompletionItemKind::Snippet,
                InsertTextFormat::Snippet,
                None,
            ),
        ]);
    }
    res
}
