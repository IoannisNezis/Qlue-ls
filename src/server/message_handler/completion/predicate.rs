use indoc::indoc;
use ll_sparql_parser::ast::{AstNode, QueryUnit};

use crate::server::{
    fetch::fetch_sparql_result,
    lsp::{
        textdocument::{Range, TextEdit},
        CompletionItem, CompletionItemKind, InsertTextFormat,
    },
    Server,
};

use super::CompletionContext;

static QUERY: &str = indoc! {
"PREFIX dblp: <https://dblp.org/rdf/schema#>
 SELECT ?pred (COUNT(?pred) as ?count)  WHERE {
   ?s ?pred ?o .
 }
 GROUP BY ?pred
 ORDER BY DESC(?count)
 LIMIT 100
"
};

pub(super) async fn completions(
    server: &Server,
    context: CompletionContext,
) -> Vec<CompletionItem> {
    let maybe_prologue = QueryUnit::cast(context._tree).and_then(|qu| qu.prologue());
    match fetch_sparql_result("https://qlever.cs.uni-freiburg.de/api/dblp/", QUERY).await {
        Ok(result) => result
            .results
            .bindings
            .into_iter()
            .map(|binding| {
                let mut import_edit: Option<TextEdit> = None;
                let pred = binding.get("pred").unwrap();
                let count = binding.get("count").unwrap();
                let value = match server.shorten_uri(&pred.to_string()) {
                    Some((prefix, uri, curie)) => {
                        if maybe_prologue.as_ref().map_or(true, |prologue| {
                            prologue
                                .prefix_declarations()
                                .iter()
                                .all(|prefix_declaration| {
                                    prefix_declaration
                                        .prefix()
                                        .map_or(false, |declared_prefix| declared_prefix != prefix)
                                })
                        }) {
                            import_edit = Some(TextEdit::new(
                                Range::new(0, 0, 0, 0),
                                &format!("PREFIX {}: <{}>\n", prefix, uri),
                            ));
                        }
                        curie
                    }
                    None => format!("<{}>", pred.to_string()),
                };
                CompletionItem::new(
                    &value,
                    &format!("{} occurences", count.to_string()),
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
}
