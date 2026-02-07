use super::{CompletionEnvironment, error::CompletionError};
use crate::server::{
    Server,
    configuration::BackendConfiguration,
    lsp::{
        CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat,
        textdocument::{Range, TextEdit},
    },
};
use futures::lock::Mutex;
use ll_sparql_parser::ast::{AstNode, QueryUnit};
use std::rc::Rc;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: &CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let server = server_rc.lock().await;
    let default_backend = server.state.get_default_backend();
    let query_unit = QueryUnit::cast(environment.tree.clone());
    Ok(CompletionList {
        is_incomplete: false,
        item_defaults: None,
        items: server
            .state
            .get_all_backends()
            .into_iter()
            .filter(|backend| default_backend.is_none_or(|default| backend.name != default.name))
            .map(|backend| {
                let (prefix, import_edit) = compute_service_prefix(query_unit.as_ref(), backend);
                CompletionItem {
                    command: None,
                    label: backend.name.clone(),
                    label_details: None,
                    kind: CompletionItemKind::Value,
                    detail: Some(backend.url.clone()),
                    documentation: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: Some(prefix),
                    text_edit: None,
                    insert_text_format: Some(InsertTextFormat::PlainText),
                    additional_text_edits: import_edit,
                }
            })
            .collect(),
    })
}

/// Determines the prefix to use for a backend's SERVICE clause and, if needed,
/// generates a PREFIX declaration to insert at the top of the document.
///
/// Scans the query's existing PREFIX declarations looking for one whose IRI
/// matches the backend URL. When a match is found, the existing prefix is
/// reused and no additional text edit is produced. When no match is found (or
/// no parse tree is available), a new prefix is synthesized from the backend
/// name via [`normalize_backend_prefix`] and a text edit is returned to insert
/// the corresponding PREFIX declaration at the start of the document.
///
/// # Returns
///
/// A tuple of:
/// - The prefix string to insert into the SERVICE clause (e.g. `"dbpedia:"`)
/// - An optional text edit that adds a PREFIX declaration at line 0, col 0.
///   `None` when an existing declaration was reused.
fn compute_service_prefix(
    query_unit: Option<&QueryUnit>,
    backend: &BackendConfiguration,
) -> (String, Option<Vec<TextEdit>>) {
    if let Some(query_unit) = query_unit {
        // NOTE: try to find an existing PREFIX declaration whose IRI matches the backend URL
        if let Some(prefix_declaration) = query_unit.prologue().and_then(|prologue| {
            prologue
                .prefix_declarations()
                .into_iter()
                .find(|prefix_declaration| {
                    prefix_declaration
                        .raw_uri_prefix()
                        .is_some_and(|uri| uri == backend.url)
                })
                // INFO: extract just the prefix name (e.g. "dbpedia") from the declaration
                .and_then(|prefix_declaration| prefix_declaration.prefix())
        }) {
            // INFO: reuse the existing prefix — no text edit needed
            (format!("{}:", prefix_declaration), None)
        } else {
            // NOTE: no matching prefix found — synthesize one and produce a text edit
            let prefix = normalize_backend_prefix(&backend.name);
            let prefix_declaration = format!("PREFIX {} <{}>\n", prefix, backend.url);
            (
                prefix,
                // INFO: insert the new PREFIX declaration at the very start of the document
                // TODO: insert PREFIX declaration AFTER comments
                Some(vec![TextEdit::new(
                    Range::new(0, 0, 0, 0),
                    &prefix_declaration,
                )]),
            )
        }
    } else {
        // NOTE: no parse tree available — always synthesize a new prefix declaration
        let prefix = normalize_backend_prefix(&backend.name);
        let prefix_declaration = format!("PREFIX {} <{}>\n", prefix, backend.url);
        (
            prefix,
            Some(vec![TextEdit::new(
                Range::new(0, 0, 0, 0),
                &prefix_declaration,
            )]),
        )
    }
}

fn normalize_backend_prefix(backend_name: &str) -> String {
    format!("{}-service:", backend_name.replace(" ", "_"))
}
