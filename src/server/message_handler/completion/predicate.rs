use super::{error::CompletionError, utils::reduce_path, CompletionEnvironment};
use crate::server::{
    lsp::{errors::LSPError, CompletionList},
    message_handler::completion::{
        environment::CompletionLocation,
        utils::{dispatch_completion_query, CompletionTemplate},
    },
    Server,
};
use futures::lock::Mutex;
use ll_sparql_parser::{ast::AstNode, syntax_kind::SyntaxKind};
use std::rc::Rc;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    // NOTE: Compute template Context
    let mut template_context = environment.template_context(server_rc.clone()).await;
    template_context.insert("local_context", &local_context(&environment)?);
    log::info!("{:?}", template_context);
    dispatch_completion_query(
        server_rc,
        &environment,
        template_context,
        CompletionTemplate::PredicateCompletion,
        true,
    )
    .await
}

fn local_context(environment: &CompletionEnvironment) -> Result<String, CompletionError> {
    if let CompletionLocation::Predicate(triple) = &environment.location {
        let subject_string = triple
            .subject()
            .ok_or(CompletionError::ResolveError(format!(
                "No subject in {}",
                triple.text()
            )))?
            .text();
        if environment
            .continuations
            .contains(&SyntaxKind::PropertyListPath)
            || environment
                .continuations
                .contains(&SyntaxKind::PropertyListPathNotEmpty)
        {
            return Ok(format!("{} ?qlue_ls_entity []", subject_string));
        } else {
            let properties = triple.properties_list_path().unwrap().properties();
            if environment.continuations.contains(&SyntaxKind::VerbPath) {
                return Ok(format!("{} ?qlue_ls_entity []", triple.text()));
            } else if properties.len() == 1 {
                return reduce_path(
                    &subject_string,
                    &properties[0].verb,
                    "[]",
                    environment
                        .anchor_token
                        .as_ref()
                        .unwrap()
                        .text_range()
                        .end(),
                )
                .ok_or(CompletionError::ResolveError(
                    "Could not build path for completion query".to_string(),
                ));
            } else {
                let (last_prop, prev_prop) = properties.split_last().unwrap();
                return Ok(format!(
                    "{} {} . {}",
                    subject_string,
                    prev_prop
                        .iter()
                        .map(|prop| prop.text())
                        .collect::<Vec<_>>()
                        .join(" ; "),
                    reduce_path(
                        &subject_string,
                        &last_prop.verb,
                        "[]",
                        environment
                            .anchor_token
                            .as_ref()
                            .unwrap()
                            .text_range()
                            .end()
                    )
                    .ok_or(CompletionError::ResolveError(
                        "Could not build path for completion query".to_string(),
                    ))?
                ));
            }
        };
    }
    {
        panic!("predicate completion called for non predicate location");
    }
}
