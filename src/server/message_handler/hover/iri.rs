use std::rc::Rc;

use crate::{
    server::{
        Server,
        lsp::errors::{ErrorCode, LSPError},
        message_handler::misc::resolve_backend_at_token,
        sparql_operations::execute_query,
        uri_converter::Converter,
    },
    sparql::results::SparqlResultsBody,
};
use futures::lock::Mutex;
use ll_sparql_parser::{
    SyntaxNode, SyntaxToken,
    ast::{AstNode, Iri, Unit},
};
use tera::Context;

pub(super) async fn hover(
    server_rc: Rc<Mutex<Server>>,
    root: SyntaxNode,
    hovered_token: SyntaxToken,
) -> Result<Option<String>, LSPError> {
    let server = server_rc.lock().await;
    let iri = match hovered_token.parent_ancestors().find_map(Iri::cast) {
        Some(value) => value,
        None => return Ok(None),
    };
    if let Some(label) = server.state.label_memory.get(&iri.text()) {
        Ok(Some(label.clone()))
    } else {
        let ast = Unit::cast(root).expect("tree should be of kind QueryUnit or UpdateUnit");
        let backend =
            resolve_backend_at_token(&server, &ast, &hovered_token).ok_or(LSPError::new(
                ErrorCode::InternalError,
                "Could not determine backend for hover location",
            ))?;
        let converter = server
            .state
            .get_converter(&backend.name)
            .ok_or(LSPError::new(
                ErrorCode::InternalError,
                "Could not get uri converter",
            ))?;
        let mut context = Context::new();
        context.insert("entity", &iri.text());
        context.insert(
            "prefixes",
            &get_iri_prefix_declaration(&iri, &ast, converter)
                .map(|prefix_pair| vec![prefix_pair])
                .unwrap_or_default(),
        );
        let query = server
            .tools
            .tera
            .render(&format!("{}-hover", backend.name), &context)
            .map_err(|err| {
                tracing::error!("{}", err);
                LSPError::new(ErrorCode::InternalError, &err.to_string())
            })?;
        let method = server.state.get_backend_request_method(&backend.name);
        let sparql_response = execute_query(
            server_rc.clone(),
            backend.url,
            query,
            None,
            None,
            Some(server.settings.completion.timeout_ms),
            method,
            None,
            0,
            false,
        )
        .await
        .map_err(|_err| LSPError::new(ErrorCode::InternalError, "hover query failed"))?;

        let result = sparql_response.expect("Non-lazy request should always return a result.");

        let SparqlResultsBody::Results { bindings } = result.body else {
            tracing::error!(
                "The SPARQL result of a completion query did not contain bindings. Likely because its not a SELECT query."
            );
            return Err(LSPError::new(
                ErrorCode::InvalidParams,
                "The SPARQL result of a completion query did not contain bindings. Likely because its not a SELECT query.",
            ));
        };
        match bindings.first() {
            Some(binding) => binding
                .get("qls_label")
                .ok_or(LSPError::new(
                    ErrorCode::InternalError,
                    "No RDF literal \"qls_entity\" in result",
                ))
                .map(|rdf_term| Some(rdf_term.value().to_string())),
            None => Ok(None),
        }
    }
}

// Creates the the prefix declaration for any IRI.
// If the iri is raw (no prefix used) None is returned.
// If the iri is a prefixed name the uri prefix is resolved in this order.
// 1. Is there a prefix declaration at the top of the document declaring this prefix.
//    The first prefix found is used.
// 2. Is this prefix stored in the iri converter of the given backend.
fn get_iri_prefix_declaration(
    iri: &Iri,
    ast: &Unit,
    converter: &Converter,
) -> Option<(String, String)> {
    let used_prefix = iri.prefixed_name()?.prefix();
    let prologue_def = ast.prologue().and_then(|prologue| {
        prologue
            .prefix_declarations()
            .iter()
            .find_map(|prefix_declaration| {
                let prefix = prefix_declaration.prefix()?;
                let uri_prefix = prefix_declaration.raw_uri_prefix()?;
                (used_prefix == prefix).then_some((prefix, uri_prefix))
            })
    });
    let storred_def = converter
        .find_by_prefix(&used_prefix)
        .cloned()
        .map(|record| (used_prefix, record.uri_prefix));
    prologue_def.or(storred_def)
}
