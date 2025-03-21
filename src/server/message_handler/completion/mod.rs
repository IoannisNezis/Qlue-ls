mod context;
mod error;
mod object;
mod predicate;
mod select_binding;
mod solution_modifier;
mod start;
mod subject;

use context::{CompletionContext, CompletionLocation};
use error::to_resonse_error;

use crate::server::{
    lsp::{errors::ResponseError, CompletionRequest, CompletionResponse},
    Server,
};

pub(super) async fn handle_completion_request(
    server: &mut Server,
    request: CompletionRequest,
) -> Result<(), ResponseError> {
    let context =
        CompletionContext::from_completion_request(server, &request).map_err(to_resonse_error)?;
    server.send_message(CompletionResponse::new(
        request.get_id(),
        match context.location {
            CompletionLocation::Start => start::completions(context).await,
            CompletionLocation::SelectBinding(_) => select_binding::completions(context),
            CompletionLocation::Subject => subject::completions(context).await,
            CompletionLocation::Predicate => predicate::completions(server, context).await,
            CompletionLocation::Object => object::completions(context),
            CompletionLocation::SolutionModifier => solution_modifier::completions(context),
            _ => vec![],
        },
    ))
}

#[cfg(test)]
mod tests;
