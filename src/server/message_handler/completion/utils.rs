use ll_sparql_parser::ast::{QueryUnit, Triple};
use sparql::results::RDFTerm;
use tera::Context;

use crate::server::{
    fetch::fetch_sparql_result,
    lsp::{
        textdocument::{Position, Range, TextEdit},
        CompletionItem, CompletionItemKind,
    },
    Server,
};

use super::{context::CompletionContext, error::CompletionError};

pub(super) async fn fetch_online_completions(
    server: &Server,
    query_unit: &QueryUnit,
    backend_name: Option<&String>,
    query_template: &str,
    mut query_template_context: Context,
    range: Range,
) -> Result<Vec<CompletionItem>, CompletionError> {
    query_template_context.insert("limit", &server.settings.completion.result_size_limit);
    query_template_context.insert("offset", &0);
    let query = server
        .tools
        .tera
        .render(query_template, &query_template_context)
        .map_err(|err| CompletionError::TemplateError(query_template.to_string(), err))?;

    let backend = backend_name.ok_or(CompletionError::ResolveError(
        "Could not resolve online completion, no backend provided.".to_string(),
    ))?;
    let url = &server
        .state
        .get_backend(backend)
        .ok_or(CompletionError::ResolveError(
            "No default SPARQL backend defined".to_string(),
        ))?
        .url;
    log::debug!("Query:\n{}", query);
    let result = fetch_sparql_result(url, &query, server.settings.completion.timeout_ms)
        .await
        .map_err(|err| CompletionError::RequestError(err.message))?;
    Ok(result
        .results
        .bindings
        .into_iter()
        .enumerate()
        .map(|(idx, binding)| {
            let rdf_term = binding
                .get("qlue_ls_entity")
                .expect("Every completion query should provide a `qlue_ls_entity`");
            let (value, import_edit) = render_rdf_term(server, query_unit, rdf_term, backend_name);
            let label = binding
                .get("qlue_ls_label")
                .map_or(value.clone(), |label| label.to_string());
            let detail = binding
                .get("qlue_ls_detail")
                .map(|detail| detail.to_string())
                .or(Some(value.clone()));
            CompletionItem {
                label: format!("{} ", label),
                detail,
                sort_text: Some(format!("{:0>5}", idx)),
                insert_text: None,
                text_edit: Some(TextEdit {
                    range: range.clone(),
                    new_text: format!("{} ", value),
                }),
                kind: CompletionItemKind::Value,
                insert_text_format: None,
                additional_text_edits: import_edit.map(|edit| vec![edit]),
            }
        })
        .collect())
}

fn render_rdf_term(
    server: &Server,
    query_unit: &QueryUnit,
    rdf_term: &RDFTerm,
    backend_name: Option<&String>,
) -> (String, Option<TextEdit>) {
    match rdf_term {
        RDFTerm::Uri { value } => match server.shorten_uri(value, backend_name) {
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

/// Get the range the completion is supposed to replace
/// The context.search_term MUST be not None!
pub(super) fn get_replace_range(context: &CompletionContext) -> Range {
    Range {
        start: context.trigger_textdocument_position,
        end: Position::new(
            context.trigger_textdocument_position.line,
            context.trigger_textdocument_position.character
                - context
                    .search_term
                    .as_ref()
                    .expect("search_term should be Some")
                    .chars()
                    .fold(0, |accu, char| accu + char.len_utf16()) as u32,
        ),
    }
}

pub(super) fn get_prefix_declarations<'a>(
    server: &'a Server,
    context: &CompletionContext,
    triple: &'a Triple,
) -> Vec<(&'a String, &'a String)> {
    triple
        .used_prefixes()
        .into_iter()
        .filter_map(|prefix| {
            context.backend.as_ref().and_then(|backend| {
                server
                    .state
                    .get_converter(backend)
                    .and_then(|converter| converter.find_by_prefix(&prefix).ok())
            })
        })
        .map(|record| (&record.prefix, &record.uri_prefix))
        .collect()
}
