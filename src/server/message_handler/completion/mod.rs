mod environment;
mod error;
mod handler;
mod transformer;
mod utils;
use environment::{CompletionEnvironment, CompletionLocation};
use error::{CompletionError, to_lsp_error};
use futures::lock::Mutex;
use std::rc::Rc;
use tracing::instrument;

use crate::server::{
    Server,
    lsp::{
        CompletionList, CompletionRequest, CompletionResponse, CompletionTriggerKind,
        errors::LSPError,
    },
    message_handler::completion::transformer::{
        CompletionTransformer, ObjectSuffixTransformer, PlainTextTransformer, SemicolonTransformer,
    },
};

#[instrument(skip(server_rc,request), fields(id = %request.get_id(), uri = %request.params.base.text_document.uri))]
pub(super) async fn handle_completion_request(
    server_rc: Rc<Mutex<Server>>,
    request: CompletionRequest,
) -> Result<(), LSPError> {
    let env = CompletionEnvironment::from_completion_request(server_rc.clone(), &request)
        .await
        .map_err(to_lsp_error)?;
    // tracing::debug!("Completion env:\n{env}");

    let mut completion_list = if (env.trigger_kind == CompletionTriggerKind::TriggerCharacter
        && env.trigger_character.as_ref().is_some_and(|tc| tc == "?")
        || env
            .search_term
            .as_ref()
            .is_some_and(|search_term| search_term.starts_with("?")))
        && !matches!(
            env.location,
            CompletionLocation::SelectBinding(_) | CompletionLocation::GroupCondition(_)
        ) {
        Some(
            handler::variable::completions(server_rc.clone(), &env)
                .await
                .map_err(to_lsp_error)?,
        )
    } else {
        let variable_completions = matches!(
            env.location,
            CompletionLocation::Subject
                | CompletionLocation::Predicate(_)
                | CompletionLocation::Object(_)
                | CompletionLocation::BlankNodeProperty(_)
                | CompletionLocation::BlankNodeObject(_)
        )
        .then_some(
            handler::variable::completions(server_rc.clone(), &env)
                .await
                .ok(),
        )
        .flatten();
        let completion_list = (env.location != CompletionLocation::Unknown).then_some(
            match env.location {
                CompletionLocation::Start => handler::start::completions(&env).await,
                CompletionLocation::SelectBinding(_) => handler::select_binding::completions(&env),
                CompletionLocation::Subject => {
                    handler::subject::completions(server_rc.clone(), &env).await
                }
                CompletionLocation::Predicate(_) => {
                    handler::predicate::completions(server_rc.clone(), &env).await
                }
                CompletionLocation::Object(_) => {
                    handler::object::completions(server_rc.clone(), &env).await
                }
                CompletionLocation::SolutionModifier => {
                    handler::solution_modifier::completions(&env)
                }
                CompletionLocation::Graph => handler::graph::completions(&env),
                CompletionLocation::BlankNodeProperty(_) => {
                    handler::blank_node_property::completions(server_rc.clone(), &env).await
                }
                CompletionLocation::BlankNodeObject(_) => {
                    handler::blank_node_object::completions(server_rc.clone(), &env).await
                }
                CompletionLocation::ServiceUrl => {
                    handler::service_url::completions(server_rc.clone(), &env).await
                }
                CompletionLocation::OrderCondition => {
                    handler::order_condition::completions(server_rc.clone(), &env).await
                }
                CompletionLocation::GroupCondition(_) => {
                    handler::group_condition::completions(server_rc.clone(), &env).await
                }
                CompletionLocation::InlineData(..) => {
                    handler::inline_data::completions(server_rc.clone(), &env).await
                }
                CompletionLocation::BuiltInCall => handler::built_in_call::completions().await,
                ref location => Err(CompletionError::Localization(format!(
                    "Unknown location \"{:?}\"",
                    location
                ))),
            }
            .map_err(to_lsp_error)?,
        );
        merge_completions(completion_list, variable_completions)
    }
    .unwrap_or_default();

    let server = server_rc.lock().await;
    if let Some(transformer) = ObjectSuffixTransformer::try_from_env(&server, &env) {
        transformer.transform(&mut completion_list);
    }
    if let Some(transformer) = SemicolonTransformer::try_from_env(&server, &env) {
        transformer.transform(&mut completion_list);
    }
    // Not every client supports `itemDefaults` or snippets (e.g. Eglot).
    if !client_supports_item_default(&server, "insertTextFormat") {
        completion_list.resolve_insert_text_format();
    }
    if let Some(transformer) = PlainTextTransformer::try_from_env(&server) {
        transformer.transform(&mut completion_list);
    }

    server.send_message(CompletionResponse::new(request.get_id(), completion_list))
}

/// Whether the client announced support for `property` in
/// `CompletionList.itemDefaults` (LSP 3.17).
fn client_supports_item_default(server: &Server, property: &str) -> bool {
    server
        .client_capabilities
        .as_ref()
        .and_then(|capabilities| capabilities.text_document.as_ref())
        .and_then(|text_document| text_document.completion.as_ref())
        .and_then(|completion| completion.completion_list.as_ref())
        .and_then(|completion_list| completion_list.item_defaults.as_ref())
        .is_some_and(|item_defaults| item_defaults.iter().any(|name| name == property))
}

fn merge_completions(
    completion_list: Option<CompletionList>,
    variable_completions: Option<CompletionList>,
) -> Option<CompletionList> {
    match (completion_list, variable_completions) {
        (None, None) => None,
        (None, Some(list)) | (Some(list), None) => Some(list),
        (Some(mut list1), Some(mut list2)) => {
            // A list's default insertTextFormat only applies to its own items,
            // and the merged list can't carry two different defaults. So this
            // is needed even for clients that support `itemDefaults`.
            list1.resolve_insert_text_format();
            list2.resolve_insert_text_format();
            list1.items.extend(list2.items);
            Some(list1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::merge_completions;
    use crate::server::lsp::{
        CompletionItem, CompletionItemKind, CompletionList, InsertTextFormat, ItemDefaults,
    };

    fn list(insert_text_format: InsertTextFormat, insert_text: &str) -> CompletionList {
        CompletionList {
            is_incomplete: false,
            item_defaults: Some(ItemDefaults {
                commit_characters: None,
                edit_range: None,
                insert_text_format: Some(insert_text_format),
                insert_text_mode: None,
                data: None,
            }),
            items: vec![CompletionItem::new(
                insert_text,
                None,
                None,
                insert_text,
                CompletionItemKind::Snippet,
                None,
            )],
        }
    }

    #[test]
    fn merge_keeps_default_insert_text_format_per_list() {
        let merged = merge_completions(
            Some(list(InsertTextFormat::Snippet, "FILTER ($0)")),
            Some(list(InsertTextFormat::PlainText, "?price$")),
        )
        .unwrap();

        assert_eq!(merged.item_defaults, None);
        assert_eq!(
            merged.items[0].insert_text_format,
            Some(InsertTextFormat::Snippet)
        );
        assert_eq!(
            merged.items[1].insert_text_format,
            Some(InsertTextFormat::PlainText)
        );
    }
}
