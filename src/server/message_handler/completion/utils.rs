use ll_sparql_parser::ast::QueryUnit;
use sparql::results::RDFTerm;
use tera::Context;

use crate::server::{
    fetch::fetch_sparql_result,
    lsp::{
        errors::{ErrorCode, LSPError},
        textdocument::{Range, TextEdit},
        CompletionItem, CompletionItemKind, InsertTextFormat,
    },
    Server,
};

pub(super) async fn fetch_online_completions(
    server: &Server,
    query_unit: &QueryUnit,
    query_template: &str,
    query_template_context: Context,
) -> Result<Vec<CompletionItem>, LSPError> {
    let query = server
        .tools
        .tera
        .render(query_template, &query_template_context)
        .map_err(|err| {
            LSPError::new(
                ErrorCode::InternalError,
                &format!(
                    "Could not render query template \"{}\"\n{:?}",
                    query_template, err
                ),
            )
        })?;
    let url = match &server.state.backend {
        Some(backend) => Ok(&backend.url),
        None => Err(LSPError::new(
            ErrorCode::InternalError,
            "No SPARQL backend defined",
        )),
    }?;

    log::info!("canary2");
    match fetch_sparql_result(url, &query).await {
        Ok(result) => Ok(result
            .results
            .bindings
            .into_iter()
            .enumerate()
            .map(|(idx, binding)| {
                log::info!("canary3");
                let value = binding
                    .get("qlue_ls_value")
                    .expect("Every completion query should provide a `qlue_ls_value`");
                let label = binding.get("qlue_ls_label").unwrap_or(value);
                let detail = binding.get("qlue_ls_detail");
                let (value, import_edit) = compress_rdf_term(server, query_unit, value);

                log::info!("canary4");
                CompletionItem::new(
                    &format!("{} ", label),
                    detail.map(|x| x.to_string()),
                    Some(format!("{:0>5}", idx)),
                    &value.to_string(),
                    CompletionItemKind::Value,
                    InsertTextFormat::PlainText,
                    import_edit.map(|edit| vec![edit]),
                )
            })
            .collect()),
        Err(err) => Err(LSPError::new(
            ErrorCode::InternalError,
            &format!("Completion query failed:\n {:?}", err),
        )),
    }
}

pub(super) fn compress_rdf_term(
    server: &Server,
    query_unit: &QueryUnit,
    rdf_term: &RDFTerm,
) -> (String, Option<TextEdit>) {
    match rdf_term {
        RDFTerm::Uri { ref value } => match server.shorten_uri(value) {
            Some((prefix, uri, curie)) => {
                let prefix_decl_edit = if query_unit.prologue().as_ref().map_or(true, |prologue| {
                    prologue
                        .prefix_declarations()
                        .iter()
                        .all(|prefix_declaration| {
                            prefix_declaration
                                .prefix()
                                .map_or(false, |declared_prefix| declared_prefix != prefix)
                        })
                }) {
                    Some(TextEdit::new(
                        Range::new(0, 0, 0, 0),
                        &format!("PREFIX {}: <{}>\n", prefix, uri),
                    ))
                } else {
                    None
                };
                (curie, prefix_decl_edit)
            }
            None => (rdf_term.to_string(), None),
        },
        _ => (rdf_term.to_string(), None),
    }
}
