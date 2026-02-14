mod environment;
mod error;
mod handler;
mod transformer;
mod utils;
use environment::{CompletionEnvironment, CompletionLocation};
use error::{CompletionError, to_lsp_error};
use futures::lock::Mutex;
use std::rc::Rc;

use crate::server::{
    Server,
    lsp::{
        CompletionList, CompletionRequest, CompletionResponse, CompletionTriggerKind,
        errors::LSPError, textdocument::Range,
    },
    message_handler::completion::transformer::{
        CompletionTransformer, ObjectSuffixTransformer, SemicolonTransformer,
    },
};

pub(super) async fn handle_completion_request(
    server_rc: Rc<Mutex<Server>>,
    request: CompletionRequest,
) -> Result<(), LSPError> {
    let mut env = CompletionEnvironment::from_completion_request(server_rc.clone(), &request)
        .await
        .map_err(to_lsp_error)?;
    // log::info!("Completion env:\n{}", env);

    let mut completion_list = if env.trigger_kind == CompletionTriggerKind::TriggerCharacter
        && env.trigger_character.as_ref().is_some_and(|tc| tc == "?")
        || env
            .search_term
            .as_ref()
            .is_some_and(|search_term| search_term.starts_with("?"))
    {
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
                CompletionLocation::FilterConstraint | CompletionLocation::GroupCondition => {
                    env.replace_range = Range::empty(env.trigger_textdocument_position);
                    handler::variable::completions(server_rc.clone(), &env).await
                }
                CompletionLocation::OrderCondition => {
                    handler::order_condition::completions(server_rc.clone(), &env).await
                }
                CompletionLocation::InlineData(..) => {
                    handler::inline_data::completions(server_rc.clone(), &env).await
                }
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
