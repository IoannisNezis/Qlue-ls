use crate::server::{
    fetch::fetch_sparql_result,
    lsp::errors::{ErrorCode, LSPError},
    message_handler::misc::resolve_backend,
    Server,
};
use ll_sparql_parser::{
    ast::{AstNode, Iri, QueryUnit},
    SyntaxNode, SyntaxToken,
};
use tera::Context;

pub(super) async fn hover(
    server: &Server,
    root: SyntaxNode,
    hovered_token: SyntaxToken,
) -> Result<Option<String>, LSPError> {
    let iri = match hovered_token.parent_ancestors().find_map(Iri::cast) {
        Some(value) => value,
        None => return Ok(None),
    };
    let mut context = Context::new();
    context.insert("entity", &iri.text());
    let query_unit = QueryUnit::cast(root).ok_or(LSPError::new(
        ErrorCode::InternalError,
        "Hover is currently only supported for Query operations",
    ))?;
    let backend = resolve_backend(server, &query_unit, &hovered_token).ok_or(LSPError::new(
        ErrorCode::InternalError,
        "Could not determine backend for hover location",
    ))?;
    if let Some(label) = server.state.label_memory.get(&iri.text()) {
        Ok(Some(label.clone()))
    } else {
        let converter = server
            .state
            .get_converter(&backend.name)
            .ok_or(LSPError::new(
                ErrorCode::InternalError,
                "Could not get uri converter",
            ))?;
        context.insert(
            "prefixes",
            &iri.prefixed_name()
                .and_then(|prefixed_name| converter.find_by_prefix(&prefixed_name.prefix()).ok())
                .map(|record| vec![(record.prefix.clone(), record.uri_prefix.clone())])
                .unwrap_or_default(),
        );
        let query = server
            .tools
            .tera
            .render(&format!("{}-hover", backend.name), &context)
            .map_err(|err| {
                log::error!("{}", err);
                LSPError::new(ErrorCode::InternalError, &err.to_string())
            })?;
        let method = server.state.get_backend_request_method(&backend.name);
        let sparql_response = fetch_sparql_result(
            &backend.url,
            &query,
            server.settings.completion.timeout_ms,
            method,
        )
        .await
        .map_err(|_err| LSPError::new(ErrorCode::InternalError, "hover query failed"))?;
        match sparql_response.results.bindings.first() {
            Some(binding) => binding
                .get("qlue_ls_label")
                .ok_or(LSPError::new(
                    ErrorCode::InternalError,
                    "No RDF literal \"qlue_ls_entity\" in result",
                ))
                .map(|rdf_term| Some(rdf_term.value().to_string())),
            None => Ok(None),
        }
    }
}
