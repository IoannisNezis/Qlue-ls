mod blank_node_object;
mod blank_node_property;
mod environment;
mod error;
mod graph;
mod object;
mod order_condition;
mod predicate;
mod select_binding;
mod service_url;
mod solution_modifier;
mod start;
mod subject;
mod utils;
mod variable;

use std::rc::Rc;

use environment::{CompletionEnvironment, CompletionLocation};
use error::{CompletionError, to_lsp_error};
use futures::lock::Mutex;

use crate::server::{
    Server,
    lsp::{
        CompletionItem, CompletionList, CompletionRequest, CompletionResponse,
        CompletionTriggerKind, InsertTextFormat, errors::LSPError,
    },
    state::ClientType,
};

pub(super) async fn handle_completion_request(
    server_rc: Rc<Mutex<Server>>,
    request: CompletionRequest,
) -> Result<(), LSPError> {
    let env = CompletionEnvironment::from_completion_request(server_rc.clone(), &request)
        .await
        .map_err(to_lsp_error)?;
    // log::info!("Completion env:\n{}", env);

    let is_object_location = matches!(env.location, CompletionLocation::Object(_));
    let line_indentation = env.line_indentation.clone();

    let completion_list = if env.trigger_kind == CompletionTriggerKind::TriggerCharacter
        && env.trigger_character.as_ref().is_some_and(|tc| tc == "?")
        || env
            .search_term
            .as_ref()
            .is_some_and(|search_term| search_term.starts_with("?"))
    {
        Some(
            variable::completions(server_rc.clone(), &env)
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
            variable::completions_transformed(server_rc.clone(), &env)
                .await
                .ok(),
        )
        .flatten();

        let completion_list = if env.location == CompletionLocation::Unknown {
            None
        } else {
            Some(
                match env.location {
                    CompletionLocation::Start => start::completions(env).await,
                    CompletionLocation::SelectBinding(_) => select_binding::completions(env),
                    CompletionLocation::Subject => {
                        subject::completions(server_rc.clone(), env).await
                    }
                    CompletionLocation::Predicate(_) => {
                        predicate::completions(server_rc.clone(), env).await
                    }
                    CompletionLocation::Object(_) => {
                        object::completions(server_rc.clone(), env).await
                    }
                    CompletionLocation::SolutionModifier => solution_modifier::completions(env),
                    CompletionLocation::Graph => graph::completions(env),
                    CompletionLocation::BlankNodeProperty(_) => {
                        blank_node_property::completions(server_rc.clone(), env).await
                    }
                    CompletionLocation::BlankNodeObject(_) => {
                        blank_node_object::completions(server_rc.clone(), env).await
                    }
                    CompletionLocation::ServiceUrl => {
                        service_url::completions(server_rc.clone(), env).await
                    }
                    CompletionLocation::FilterConstraint | CompletionLocation::GroupCondition => {
                        variable::completions_transformed(server_rc.clone(), &env).await
                    }
                    CompletionLocation::OrderCondition => {
                        order_condition::completions(server_rc.clone(), env).await
                    }
                    location => Err(CompletionError::Localization(format!(
                        "Unknown location \"{:?}\"",
                        location
                    ))),
                }
                .map_err(to_lsp_error)?,
            )
        };

        merge_completions(completion_list, variable_completions)
    };

    let server = server_rc.lock().await;

    // Apply object suffix (trailing ` .\n` with indentation and cursor position)
    let completion_list = if is_object_location {
        completion_list.map(|mut list| {
            for item in list.items.iter_mut() {
                apply_object_suffix(
                    item,
                    server
                        .state
                        .client_Type
                        .as_ref()
                        .is_some_and(|client_type| matches!(client_type, ClientType::Monaco))
                        .then_some("")
                        .unwrap_or(&line_indentation),
                );
            }
            list
        })
    } else {
        completion_list
    };

    server.send_message(CompletionResponse::new(request.get_id(), completion_list))
}

fn merge_completions(
    completion_list: Option<CompletionList>,
    variable_completions: Option<CompletionList>,
) -> Option<CompletionList> {
    match (completion_list, variable_completions) {
        (None, None) => None,
        (None, Some(list)) | (Some(list), None) => Some(list),
        (Some(mut list1), Some(list2)) => {
            list1.items.extend(list2.items);
            Some(list1)
        }
    }
}

fn apply_object_suffix(item: &mut CompletionItem, indent: &str) {
    // Handle text_edit (used by online completions)
    if let Some(ref mut text_edit) = item.text_edit {
        text_edit.new_text = format!("{} .\n{indent}$0", text_edit.new_text.trim_end());
    }
    // Handle insert_text (used by variable completions)
    if let Some(ref mut insert_text) = item.insert_text {
        *insert_text = format!("{} .\n{indent}$0", insert_text.trim_end());
    }
    item.insert_text_format = Some(InsertTextFormat::Snippet);
}
