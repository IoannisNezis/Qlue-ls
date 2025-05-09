use crate::server::{
    fetch::fetch_sparql_result,
    lsp::errors::{ErrorCode, LSPError},
    Server,
};
use ll_sparql_parser::{
    ast::{AstNode, Iri},
    SyntaxToken,
};
use tera::Context;

pub(super) async fn hover(server: &Server, token: SyntaxToken) -> Result<Option<String>, LSPError> {
    let iri = match token.parent_ancestors().find_map(Iri::cast) {
        Some(value) => value,
        None => return Ok(None),
    };
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
    let sparql_response =
        fetch_sparql_result(backend_url, &query, server.settings.completion.timeout_ms).await?;
    match sparql_response.results.bindings.first() {
        Some(binding) => binding
            .get("qlue_ls_entity")
            .ok_or(LSPError::new(
                ErrorCode::InternalError,
                "No RDF literal \"qlue_ls_entity\" in result",
            ))
            .map(|rdf_term| Some(rdf_term.to_string())),
        None => Ok(None),
    }
}
