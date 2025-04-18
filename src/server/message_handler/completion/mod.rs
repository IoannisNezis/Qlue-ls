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

use context::{CompletionContext, CompletionLocation};
use error::{to_lsp_error, CompletionError};

use crate::server::{
    lsp::{errors::LSPError, CompletionRequest, CompletionResponse, CompletionTriggerKind},
    Server,
};

pub(super) async fn handle_completion_request(
    server: &mut Server,
    request: CompletionRequest,
) -> Result<(), LSPError> {
    let context =
        CompletionContext::from_completion_request(server, &request).map_err(to_lsp_error)?;
    if context.location == CompletionLocation::Unknown {
        server.send_message(CompletionResponse::new(request.get_id(), None))
    } else {
        server.send_message(CompletionResponse::new(
            request.get_id(),
            Some(
                if context.trigger_kind == CompletionTriggerKind::TriggerCharacter
                    && context
                        .trigger_character
                        .as_ref()
                        .map_or(false, |tc| tc == "?")
                    || context
                        .search_term
                        .as_ref()
                        .map_or(false, |search_term| search_term.starts_with("?"))
                {
                    variable::completions(context)
                } else {
                    match context.location {
                        CompletionLocation::Start => start::completions(context).await,
                        CompletionLocation::SelectBinding(_) => {
                            select_binding::completions(context)
                        }
                        CompletionLocation::Subject => subject::completions(server, context).await,
                        CompletionLocation::Predicate(_) => {
                            predicate::completions(server, context).await
                        }
                        CompletionLocation::Object(_) => object::completions(server, context).await,
                        CompletionLocation::SolutionModifier => {
                            solution_modifier::completions(context)
                        }
                        CompletionLocation::Graph => graph::completions(context),
                        CompletionLocation::BlankNodeProperty(_) => {
                            blank_node_property::completions(server, context).await
                        }
                        CompletionLocation::ServiceUrl => service_url::completions(server),
                        location => Err(CompletionError::LocalizationError(format!(
                            "Unknown location \"{:?}\"",
                            location
                        ))),
                    }
                }
                .map_err(to_lsp_error)?,
            ),
        ))
    }
}
