use super::{error::CompletionError, utils::reduce_path, CompletionEnvironment};
use crate::server::{
    lsp::CompletionList,
    message_handler::completion::{
        environment::CompletionLocation,
        utils::{dispatch_completion_query, CompletionTemplate},
    },
    Server,
};
use futures::{channel::oneshot, lock::Mutex};
use ll_sparql_parser::{ast::AstNode, syntax_kind::SyntaxKind};
use std::rc::Rc;
use tera::Context;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    // NOTE: Compute template Context
    let mut template_context = environment.template_context().await;
    template_context.extend(local_template_context(&environment)?);

    let (sender, receiver) = oneshot::channel::<CompletionList>();

    let server_rc_1 = server_rc.clone();
    let template_context_1 = template_context.clone();
    let environment_1 = environment.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match dispatch_completion_query(
            server_rc_1,
            &environment_1,
            template_context_1,
            CompletionTemplate::PredicateCompletion,
            true,
        )
        .await
        {
            Ok(res) => {
                if let Err(_err) = sender.send(res) {
                    // NOTE: This should happen if the context sensitive completion succeeds first.
                }
            }
            Err(err) => {
                log::error!("Context insensitive completion query failed:\n{:?}", err);
            }
        };
    });

    match dispatch_completion_query(
        server_rc,
        &environment,
        template_context,
        CompletionTemplate::PredicateCompletionContextSensitive,
        true,
    )
    .await
    {
        Ok(res) => Ok(res),
        Err(err) => {
            log::error!("Context sensitive completion query failed:\n{:?}", err);
            receiver.await.map_err(|_e| err)
        }
    }
}

fn local_template_context(environment: &CompletionEnvironment) -> Result<Context, CompletionError> {
    let mut template_context = Context::new();
    if let CompletionLocation::Predicate(triple) = &environment.location {
        let subject_string = triple
            .subject()
            .ok_or(CompletionError::Resolve(format!(
                "No subject in {}",
                triple.text()
            )))?
            .text();
        template_context.insert("subject", &subject_string);
        if environment
            .continuations
            .contains(&SyntaxKind::PropertyListPath)
            || environment
                .continuations
                .contains(&SyntaxKind::PropertyListPathNotEmpty)
        {
            template_context.insert(
                "local_context",
                &format!("{} ?qlue_ls_entity []", subject_string),
            );
        } else {
            let properties = triple
                .properties_list_path()
                .ok_or(CompletionError::Resolve(format!(
                    "Could not find properties list in triple: \"{}\"",
                    triple.text()
                )))?
                .properties();
            if properties.is_empty() {
                template_context.insert(
                    "local_context",
                    &format!("{} ?qlue_ls_entity []", triple.text()),
                );
            } else {
                let (last_prop, prev_prop) = properties.split_last().unwrap();
                let mut context = environment.context.clone().unwrap_or_default();
                if environment.anchor_token.as_ref().is_some_and(|anchor| {
                    anchor.text_range().start() < last_prop.text_range().end()
                }) {
                    // NOTE: The completion was triggerd within the last property
                    if !prev_prop.is_empty() {
                        context.raw_inject = format!(
                            "{} {}",
                            subject_string,
                            prev_prop
                                .iter()
                                .map(|prop| prop.text())
                                .collect::<Vec<_>>()
                                .join(" ; ")
                        );
                    }

                    template_context.insert(
                        "local_context",
                        &reduce_path(
                            &subject_string,
                            &last_prop.verb,
                            "[]",
                            environment
                                .anchor_token
                                .as_ref()
                                .unwrap()
                                .text_range()
                                .end(),
                        )
                        .ok_or(CompletionError::Resolve(
                            "Could not build path for completion query".to_string(),
                        ))?,
                    );
                } else {
                    // NOTE: The completion was triggerd after the last property
                    context.raw_inject = format!(
                        "{} {}",
                        subject_string,
                        properties
                            .iter()
                            .map(|prop| prop.text())
                            .collect::<Vec<_>>()
                            .join(" ; ")
                    );
                    template_context.insert(
                        "local_context",
                        &format!("{} ?qlue_ls_entity []", subject_string),
                    );
                }
                template_context.insert("context", &context);
            }
        };
    } else {
        panic!("predicate completion called for non predicate location");
    }
    Ok(template_context)
}
