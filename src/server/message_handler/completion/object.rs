use indoc::indoc;
use lazy_static::lazy_static;
use ll_sparql_parser::ast::{AstNode, QueryUnit, SelectQuery};
use tera::{Context, Tera};

use crate::server::{
    fetch::fetch_sparql_result,
    lsp::{CompletionItem, CompletionItemKind, CompletionTriggerKind, InsertTextFormat},
    message_handler::completion::context::CompletionLocation,
    Server,
};

use super::{utils::compress_rdf_term, CompletionContext};

lazy_static! {
    static ref QUERY_TEMPATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template(
            "object_query.rq.template",
            indoc! {
               "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
                PREFIX dblp: <https://dblp.org/rdf/schema#>
                PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
                {% for prefix in prefixes %}
                    PREFIX {{prefix.0}}: <{{prefix.1}}>
                {% endfor %}
                SELECT ?entity ?name ?count  WHERE {
                  {
                    SELECT ?entity (COUNT(?entity) AS ?count) WHERE {
                        {{context}} ?entity
                    }
                    GROUP BY ?entity
                  }
                  OPTIONAL {
                    ?entity dblp:creatorName ?creatorname .
                  }
                  OPTIONAL {
                    ?entity rdfs:label ?label .
                  }
                  BIND (COALESCE(?creatorname, ?label, ?entity) AS ?name)
                }
                ORDER BY DESC(?count)
                LIMIT 100"
            },
        )
        .expect("Template should be valid");
        tera
    };
}

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
        let triples_string = triple.syntax().text().to_string();
        let (inject_context, search_term) = triples_string.split_at(
            (context.anchor_token.unwrap().text_range().end()
                - triple.syntax().text_range().start())
            .into(),
        );
        let mut template_context = Context::new();
        template_context.insert("prefixes", &prefix_declarations);
        template_context.insert("context", &inject_context);
        let query = QUERY_TEMPATES
            .render("object_query.rq.template", &template_context)
            .expect("Template should render");
        let query_unit = QueryUnit::cast(context.tree).unwrap();
        if let Some(backend) = &server.state.backend {
            match fetch_sparql_result(&backend.url, &query).await {
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
                            &value,
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
