use crate::server::{
    fetch::fetch_sparql_result,
    lsp::errors::{ErrorCode, LSPError},
    Server,
};
use ll_sparql_parser::{
    ast::{AstNode, Iri},
    SyntaxToken,
};
use sparql::results::RDFTerm;
use tera::Context;

pub(super) async fn hover(server: &Server, token: SyntaxToken) -> Result<String, LSPError> {
    let iri = token
        .parent_ancestors()
        .find_map(Iri::cast)
        .ok_or(LSPError::new(
            ErrorCode::InternalError,
            "Could not find iri node",
        ))?;
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
        .render("hover_iri.rq", &context)
        .map_err(|err| {
            log::error!("{}", err);
            LSPError::new(ErrorCode::InternalError, &err.to_string())
        })?;
    let backend_url = &server
        .state
        .get_default_backend()
        .ok_or(LSPError::new(
            ErrorCode::InternalError,
            "Could not resolve backend url",
        ))?
        .url;
    let result = fetch_sparql_result(backend_url, &query).await?;
    if let Some(RDFTerm::Literal {
        value,
        lang: _lang,
        datatype: _datatype,
    }) = result
        .results
        .bindings
        .first()
        .and_then(|value| value.get("qlue_ls_value"))
    {
        Ok(value.to_owned())
    } else {
        Err(LSPError::new(
            ErrorCode::InternalError,
            "No RDF literal \"qlue_ls_value\" in result",
        ))
    }
}
