mod context;
mod error;
mod snippets;
mod utils;

use context::{CompletionContext, CompletionLocation};
use error::to_resonse_error;
use log::{error, warn};
use snippets::{get_not_tripples_snippets, get_solution_mod_snippets, get_start_snippets};

use crate::server::{
    anaysis::get_all_variables,
    lsp::{
        errors::{ErrorCode, ResponseError},
        CompletionItem, CompletionItemKind, CompletionRequest, CompletionResponse,
        CompletionTriggerKind, InsertTextFormat,
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
        // Completion was triggered by typing an trigger character
        // NOTE: The Trigger character is "?"
        CompletionTriggerKind::TriggerCharacter  => Ok(
            CompletionResponse::new(request.get_id(), collect_completions_triggered(server, &request)?)
        )
        ,
        // Completion was triggered by typing an identifier (24x7 code complete),
        // manual invocation (e.g Ctrl+Space) or via API.
        CompletionTriggerKind::Invoked  => Ok(
            CompletionResponse::new( request.get_id(), collect_completions(completion_context.location,server, &request)?),
        ),
        CompletionTriggerKind::TriggerForIncompleteCompletions => {
            error!("Completion was triggered by \"TriggerForIncompleteCompetions\", this is not implemented yet");
            Err(ResponseError::new(ErrorCode::InvalidRequest, "Completion was triggered by \"TriggerForIncompleteCompetions\", this is not implemented yet"))
        }
    }
}

fn variable_completions(
    server: &Server,
    request: &CompletionRequest,
    triggered: bool,
) -> Result<impl Iterator<Item = CompletionItem>, ResponseError> {
    Ok(get_all_variables(
        &server.state,
        &request.get_text_position().text_document.uri,
    )?
    .into_iter()
    .map(move |variable| {
        CompletionItem::new(
            &variable,
            "variable",
            match triggered {
                true => &variable[1..],
                false => &variable,
            },
            CompletionItemKind::Snippet,
            InsertTextFormat::Snippet,
        )
    }))
}

fn graph_pattern_not_triples_completions(
    _server: &Server,
    _request: &CompletionRequest,
) -> Result<impl Iterator<Item = CompletionItem>, ResponseError> {
    Ok(get_not_tripples_snippets().into_iter())
}

fn collect_completions_triggered(
    server: &Server,
    request: &CompletionRequest,
) -> Result<Vec<CompletionItem>, ResponseError> {
    let trigger_character =
        request
            .params
            .context
            .trigger_character
            .to_owned()
            .ok_or(ResponseError::new(
                ErrorCode::InvalidParams,
                "triggered completion request has no trigger character",
            ))?;
    Ok(match trigger_character.as_str() {
        "?" => variable_completions(server, &request, true)?.collect(),
        other => {
            warn!(
                "Completion request triggered by unknown trigger character: \"{}\"",
                other
            );
            vec![]
        }
    })
}

fn collect_completions(
    location: CompletionLocation,
    server: &Server,
    request: &CompletionRequest,
) -> Result<Vec<CompletionItem>, ResponseError> {
    Ok(match location {
        CompletionLocation::Start => get_start_snippets(),
        CompletionLocation::Predicate => {
            vec![CompletionItem::new(
                "predicate filler",
                "Hier könnte ihre predicate completion stehen",
                "<predicate> ",
                CompletionItemKind::Value,
                InsertTextFormat::PlainText,
            )]
        }
        CompletionLocation::Object => {
            vec![CompletionItem::new(
                "object filler",
                "Hier könnte ihre object completion stehen",
                "<object> ",
                CompletionItemKind::Value,
                InsertTextFormat::PlainText,
            )]
        }
        CompletionLocation::TripleOrNotTriple => variable_completions(server, request, false)?
            .chain(graph_pattern_not_triples_completions(server, request)?)
            .collect(),
        CompletionLocation::End => get_solution_mod_snippets(),
        _ => vec![],
    })
}

#[cfg(test)]
mod tests;
