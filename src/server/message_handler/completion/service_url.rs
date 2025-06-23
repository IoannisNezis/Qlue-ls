use super::{error::CompletionError, CompletionEnvironment};
use crate::server::{
    lsp::{
        textdocument::{Range, TextEdit},
        Backend, CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat,
    },
    Server,
};
use futures::lock::Mutex;
use ll_sparql_parser::ast::{AstNode, QueryUnit};
use std::rc::Rc;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let server = server_rc.lock().await;
    let default_backend = server.state.get_default_backend();
    let query_unit = QueryUnit::cast(environment.tree);
    Ok(CompletionList {
        is_incomplete: false,
        item_defaults: None,
        items: server
            .state
            .get_all_backends()
            .into_iter()
            .filter(|backend| default_backend.is_none_or(|default| backend.name != default.name))
            .map(|backend| {
                let (prefix, import_edit) = backend_prefix(query_unit.as_ref(), backend);
                CompletionItem {
                    command: None,
                    label: backend.name.clone(),
                    label_details: None,
                    kind: CompletionItemKind::Value,
                    detail: Some(backend.url.clone()),
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

fn backend_prefix(
    query_unit: Option<&QueryUnit>,
    backend: &Backend,
) -> (String, Option<Vec<TextEdit>>) {
    if let Some(query_unit) = query_unit {
        if let Some(prefix_declaration) = query_unit.prologue().and_then(|prologue| {
            prologue
                .prefix_declarations()
                .into_iter()
                .find(|prefix_declaration| {
                    prefix_declaration
                        .uri_prefix()
                        .is_some_and(|uri| uri.contains(&backend.url))
                })
                .and_then(|prefix_declaration| prefix_declaration.prefix())
        }) {
            (format!("{}:", prefix_declaration), None)
        } else {
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
    } else {
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
    format!("{}:", backend_name.replace(" ", "_"))
}
