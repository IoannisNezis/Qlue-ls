use ll_sparql_parser::ast::{AstNode, QueryUnit};

use crate::server::{
    fetch::fetch_sparql_result,
    lsp::{CompletionItem, CompletionItemKind, InsertTextFormat},
    message_handler::completion::context::CompletionLocation,
    Server,
};

use super::{utils::compress_rdf_term, CompletionContext};

fn query(context: &str) -> String {
    format!(
        "
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX dblp: <https://dblp.org/rdf/schema#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
SELECT ?entity ?name ?count  WHERE {{
  {{
    SELECT ?entity (COUNT(?entity) AS ?count) WHERE {{
      {} ?entity
    }}
    GROUP BY ?entity
  }}
  OPTIONAL {{
    ?entity dblp:creatorName ?creatorname .
  }}
  OPTIONAL {{
    ?entity rdfs:label ?label .
  }}
  BIND (COALESCE(?creatorname, ?label, ?entity) AS ?name)
}}
ORDER BY DESC(?count)
LIMIT 100
",
        context
    )
}

pub(super) async fn completions(
    server: &Server,
    context: CompletionContext,
) -> Vec<CompletionItem> {
    if let CompletionLocation::Object(triple) = context.location {
        let triples_string = triple.syntax().text().to_string();
        let (inject_context, search_term) = triples_string.split_at(
            (context.anchor_token.unwrap().text_range().end()
                - triple.syntax().text_range().start())
            .into(),
        );
        log::info!("{}", search_term);
        let query_unit = QueryUnit::cast(context.tree).unwrap();
        if let Some(backend) = &server.state.backend {
            match fetch_sparql_result(&backend.url, &query(inject_context)).await {
                Ok(result) => result
                    .results
                    .bindings
                    .into_iter()
                    .map(|binding| {
                        let entiy = binding.get("entity").unwrap();
                        let name = binding.get("name").unwrap();
                        // let count = binding.get("count").unwrap();
                        let (value, import_edit) = compress_rdf_term(server, &query_unit, entiy);
                        CompletionItem::new(
                            &name.to_string(),
                            &value,
                            "",
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
