use indoc::indoc;
use lazy_static::lazy_static;
use ll_sparql_parser::{
    ast::{AstNode, Iri},
    SyntaxToken,
};
use sparql::results::RDFTerm;
use tera::{Context, Tera};

use crate::server::{fetch::fetch_sparql_result, Server};

// TODO: Templates should be loaded once at server initialization or when templates get changed
// dynamcally,
// not at every hover call...
lazy_static! {
    static ref QUERY_TEMPATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_template(
            "hover_iri_query.rq",
            indoc! {
               "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
                {% if prefix %}
                PREFIX {{prefix.0}}: <{{prefix.1}}>
                {% endif %}
                SELECT ?hover WHERE {
                  {{entity}} rdfs:label ?label .
                  OPTIONAL {
                      {{entity}} rdfs:comment ?comment .
                  }
                  Bind(COALESCE(?comment, ?label) as ?hover)
                }
                LIMIT 1
               "
            },
        )
        .expect("Template should be valid");
        tera
    };
}

pub(super) async fn hover(server: &Server, token: SyntaxToken) -> Option<String> {
    let iri = token.parent_ancestors().find_map(Iri::cast)?;
    let mut context = Context::new();
    context.insert("entity", &iri.text());

    if let Some(prefixed_name) = iri.prefixed_name() {
        if let Ok(record) = server
            .tools
            .uri_converter
            .find_by_prefix(&prefixed_name.prefix())
        {
            context.insert(
                "prefix",
                &(record.prefix.clone(), record.uri_prefix.clone()),
            );
        }
    }
    let query = QUERY_TEMPATES
        .render("hover_iri_query.rq", &context)
        .expect("Template should render");
    let backend = server.state.backend.as_ref()?;
    match fetch_sparql_result(&backend.url, &query).await {
        Ok(result) => {
            if let RDFTerm::Literal {
                value,
                lang: _lang,
                datatype: _datatype,
            } = result.results.bindings.first()?.get("hover")?
            {
                Some(value.to_owned())
            } else {
                None
            }
        }
        Err(err) => {
            log::error!("{:?}", err);
            None
        }
    }
}
