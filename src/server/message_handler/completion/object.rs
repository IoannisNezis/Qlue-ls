use super::{
    error::CompletionError,
    utils::{dispatch_completion_query, CompletionTemplate},
    CompletionEnvironment,
};
use crate::server::{
    lsp::CompletionList, message_handler::completion::environment::CompletionLocation, Server,
};
use futures::{channel::oneshot, lock::Mutex};
use ll_sparql_parser::ast::AstNode;
use std::rc::Rc;
use tera::Context;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let mut template_context = environment.template_context().await;
    template_context.extend(local_template_context(&environment)?);

    let (sender, reciever) = oneshot::channel::<CompletionList>();

    let server_rc_1 = server_rc.clone();
    let template_context_1 = template_context.clone();
    let environment_1 = environment.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match dispatch_completion_query(
            server_rc_1,
            &environment_1,
            template_context_1,
            CompletionTemplate::ObjectCompletion,
            false,
        )
        .await
        {
            Ok(res) => {
                if let Err(_err) = sender.send(res) {
                    // NOTE: This should happen if the context sensitive completion succeeds first.
                }
            }
            Err(err) => {
                log::info!("Context insensitive completion query failed:\n{:?}", err);
            }
        };
    });

    match dispatch_completion_query(
        server_rc,
        &environment,
        template_context,
        CompletionTemplate::ObjectCompletionContextSensitive,
        false,
    )
    .await
    {
        Ok(res) => Ok(res),
        Err(err) => {
            log::info!("Context sensitive completion query failed:\n{:?}", err);
            reciever.await.map_err(|_e| err)
        }
    }
}

fn local_template_context(environment: &CompletionEnvironment) -> Result<Context, CompletionError> {
    let mut template_context = Context::new();

    if let CompletionLocation::Object(triple) = &environment.location {
        let subject_string = triple
            .subject()
            .ok_or(CompletionError::Resolve(format!(
                "Could not find subject in triple: \"{}\"",
                triple.text()
            )))?
            .text();

        template_context.insert("subject", &subject_string);

        let properties = triple
            .properties_list_path()
            .ok_or(CompletionError::Resolve(format!(
                "Could not find properties list in triple: \"{}\"",
                triple.text()
            )))?
            .properties();

        let (last_prop, prev_prop) = properties
            .split_last()
            .expect("There should be atleast one property, since this is a object completion");

        let mut context = environment.context.clone().unwrap_or_default();
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
            &format!(
                "{} {} ?qlue_ls_entity",
                subject_string,
                last_prop.verb.text()
            ),
        );
        template_context.insert("context", &context);
    } else {
        panic!("object completion called for non object location");
    }
    Ok(template_context)
}
