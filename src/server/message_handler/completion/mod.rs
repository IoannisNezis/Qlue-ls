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
use log::error;

use crate::server::{
    lsp::{
        errors::{ErrorCode, ResponseError},
        CompletionItem, CompletionRequest, CompletionResponse, CompletionTriggerKind,
    },
    Server,
};

pub fn handle_completion_request(
    server: &mut Server,
    request: CompletionRequest,
) -> Result<CompletionResponse, ResponseError> {
    let completion_context =
        CompletionContext::from_completion_request(server, &request).map_err(to_resonse_error)?;
    log::info!("Location: {:?}", completion_context.location);

    match completion_context.trigger_kind {
        // NOTE: Completion was triggered by typing an trigger character
        //       The Trigger character is "?"
        CompletionTriggerKind::TriggerCharacter => Ok(CompletionResponse::new(
            request.get_id(),
            // collect_completions_triggered(server, &request)?,
            Vec::new(),
        )),
        // NOTE: Completion was triggered by typing an identifier (24x7 code complete),
        //       manual invocation (e.g Ctrl+Space) or via API.
        CompletionTriggerKind::Invoked => Ok(CompletionResponse::new(
            request.get_id(),
            collect_completions(completion_context)?,
        )),
        // NOTE: Did not read into what this is for jet
        CompletionTriggerKind::TriggerForIncompleteCompletions => {
            error!("Completion was triggered by \"TriggerForIncompleteCompetions\", this is not implemented yet");
            Err(ResponseError::new(ErrorCode::InvalidRequest, "Completion was triggered by \"TriggerForIncompleteCompetions\", this is not implemented yet"))
        }
    }
}

fn collect_completions(context: CompletionContext) -> Result<Vec<CompletionItem>, ResponseError> {
    Ok(match context.location {
        CompletionLocation::Start => start::completions(context),
        CompletionLocation::SelectBinding => select_binding::completions(context),
        CompletionLocation::Predicate => predicate::completions(context),
        CompletionLocation::Object => object::completions(context),
        CompletionLocation::Subject => subject::completions(context),
        CompletionLocation::SolutionModifier => solution_modifier::completions(context),
        _ => vec![],
    })
}

// fn collect_completions_triggered(
//     server: &Server,
//     request: &CompletionRequest,
// ) -> Result<Vec<CompletionItem>, ResponseError> {
//     let trigger_character =
//         request
//             .params
//             .context
//             .trigger_character
//             .to_owned()
//             .ok_or(ResponseError::new(
//                 ErrorCode::InvalidParams,
//                 "triggered completion request has no trigger character",
//             ))?;
//     Ok(match trigger_character.as_str() {
//         "?" => variable_completions(server, &request, true)?.collect(),
//         other => {
//             warn!(
//                 "Completion request triggered by unknown trigger character: \"{}\"",
//                 other
//             );
//             vec![]
//         }
//     })
// }

#[cfg(test)]
mod tests;
