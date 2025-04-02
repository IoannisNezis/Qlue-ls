use indoc::indoc;
use lazy_static::lazy_static;
use ll_sparql_parser::{
    ast::{AstNode, Iri},
    SyntaxToken,
};
use sparql::results::RDFTerm;
use tera::{Context, Tera};

use crate::server::{fetch::fetch_sparql_result, Server};

pub(super) async fn hover(server: &Server, token: SyntaxToken) -> Option<String> {
    let iri = token.parent_ancestors().find_map(Iri::cast)?;
    let mut context = Context::new();
    context.insert("entity", &iri.text());

    // TODO: in case of a service call use different backend
    if let Some(prefixed_name) = iri.prefixed_name() {
        if let Some(record) = server
            .state
            .get_default_converter()
            .and_then(|converter| converter.find_by_prefix(&prefixed_name.prefix()).ok())
        {
            context.insert(
                "prefix",
                &(record.prefix.clone(), record.uri_prefix.clone()),
            );
        }
    }
    let query = server
        .tools
        .tera
        .render("hover_iri_query.rq", &context)
        .expect("Template should render");
    let backend_url = &server.state.get_default_backend()?.url;
    match fetch_sparql_result(backend_url, &query).await {
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
