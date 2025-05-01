mod blank_node_object;
mod blank_node_property;
mod context;
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

use context::{CompletionContext, CompletionLocation};
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
    let context = CompletionContext::from_completion_request(server_rc.clone(), &request)
        .await
        .map_err(to_lsp_error)?;

    let completion_list = if context.trigger_kind == CompletionTriggerKind::TriggerCharacter
        && context
            .trigger_character
            .as_ref()
            .map_or(false, |tc| tc == "?")
        || context
            .search_term
            .as_ref()
            .map_or(false, |search_term| search_term.starts_with("?"))
    {
        Some(variable::completions(context).map_err(to_lsp_error)?)
    } else if context.location == CompletionLocation::Unknown {
        None
    } else {
        Some(
            match context.location {
                CompletionLocation::Start => start::completions(context).await,
                CompletionLocation::SelectBinding(_) => select_binding::completions(context),
                CompletionLocation::Subject => {
                    subject::completions(server_rc.clone(), context).await
                }
                CompletionLocation::Predicate(_) => {
                    predicate::completions(server_rc.clone(), context).await
                }
                CompletionLocation::Object(_) => {
                    object::completions(server_rc.clone(), context).await
                }
                CompletionLocation::SolutionModifier => solution_modifier::completions(context),
                CompletionLocation::Graph => graph::completions(context),
                CompletionLocation::BlankNodeProperty(_) => {
                    blank_node_property::completions(server_rc.clone(), context).await
                }
                CompletionLocation::BlankNodeObject(_) => {
                    blank_node_object::completions(server_rc.clone(), context).await
                }
                CompletionLocation::ServiceUrl => service_url::completions(server_rc.clone()).await,
                CompletionLocation::FilterConstraint | CompletionLocation::GroupCondition => {
                    variable::completions_transformed(context)
                }
                // CompletionLocation::Unknown => Ok(),
                location => Err(CompletionError::LocalizationError(format!(
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
