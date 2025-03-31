mod context;
mod error;
mod graph;
mod object;
mod predicate;
mod select_binding;
mod solution_modifier;
mod start;
mod subject;
mod utils;
mod variable;

use context::{CompletionContext, CompletionLocation};
use error::to_resonse_error;

use crate::server::{
    lsp::{errors::LSPError, CompletionRequest, CompletionResponse, CompletionTriggerKind},
    Server,
};

pub(super) async fn handle_completion_request(
    server: &mut Server,
    request: CompletionRequest,
) -> Result<(), LSPError> {
    let context =
        CompletionContext::from_completion_request(server, &request).map_err(to_resonse_error)?;

    server.send_message(CompletionResponse::new(
        request.get_id(),
        if context.trigger_kind == CompletionTriggerKind::TriggerCharacter
            && context
                .trigger_character
                .as_ref()
                .map_or(false, |tc| tc == "?")
        {
            variable::completions(context)
        } else {
            match context.location {
                CompletionLocation::Start => start::completions(context).await,
                CompletionLocation::SelectBinding(_) => select_binding::completions(context),
                CompletionLocation::Subject => subject::completions(server, context).await,
                CompletionLocation::Predicate(_) => predicate::completions(server, context).await,
                CompletionLocation::Object(_) => object::completions(server, context).await,
                CompletionLocation::SolutionModifier => solution_modifier::completions(context),
                CompletionLocation::Graph => graph::completions(context),
                _ => vec![],
            }
        },
    ))
}
