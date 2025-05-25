mod blank_node_object;
mod blank_node_property;
mod environment;
mod error;
mod graph;
mod object;
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
use error::{to_lsp_error, CompletionError};
use futures::lock::Mutex;

use crate::server::{
    lsp::{errors::LSPError, CompletionRequest, CompletionResponse, CompletionTriggerKind},
    Server,
};

pub(super) async fn handle_completion_request(
    server_rc: Rc<Mutex<Server>>,
    request: CompletionRequest,
) -> Result<(), LSPError> {
    let env = CompletionEnvironment::from_completion_request(server_rc.clone(), &request)
        .await
        .map_err(to_lsp_error)?;

    let completion_list = if env.trigger_kind == CompletionTriggerKind::TriggerCharacter
        && env.trigger_character.as_ref().is_some_and(|tc| tc == "?")
        || env
            .search_term
            .as_ref()
            .is_some_and(|search_term| search_term.starts_with("?"))
    {
        Some(variable::completions(env).map_err(to_lsp_error)?)
    } else if env.location == CompletionLocation::Unknown {
        None
    } else {
        Some(
            match env.location {
                CompletionLocation::Start => start::completions(env).await,
                CompletionLocation::SelectBinding(_) => select_binding::completions(env),
                CompletionLocation::Subject => subject::completions(server_rc.clone(), env).await,
                CompletionLocation::Predicate(_) => {
                    predicate::completions(server_rc.clone(), env).await
                }
                CompletionLocation::Object(_) => object::completions(server_rc.clone(), env).await,
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
                    variable::completions_transformed(env)
                }
                location => Err(CompletionError::Localization(format!(
                    "Unknown location \"{:?}\"",
                    location
                ))),
            }
            .map_err(to_lsp_error)?,
        )
    };
    server_rc
        .lock()
        .await
        .send_message(CompletionResponse::new(request.get_id(), completion_list))
}
