use std::rc::Rc;

use futures::{channel::oneshot, lock::Mutex};
use ll_sparql_parser::ast::AstNode;

#[cfg(not(target_arch = "wasm32"))]
use tokio::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::{
    Server,
    server::{
        lsp::CompletionList,
        message_handler::completion::{
            CompletionEnvironment, CompletionError, CompletionLocation,
            utils::{CompletionTemplate, dispatch_completion_query},
        },
    },
};

pub async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: &CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let mut template_context = environment.template_context().await;
    template_context.insert("local_context", &local_context(environment));

    let (sender, receiver) = oneshot::channel::<CompletionList>();

    let server_rc_1 = server_rc.clone();
    let template_context_1 = template_context.clone();
    let environment_1 = environment.clone();
    spawn_local(async move {
        match dispatch_completion_query(
            server_rc_1,
            &environment_1,
            template_context_1,
            CompletionTemplate::ValuesCompletionContextInsensitive,
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
        environment,
        template_context,
        CompletionTemplate::ValuesCompletionContextSensitive,
        false,
    )
    .await
    {
        Ok(res) => Ok(res),
        Err(err) => {
            log::info!("Context sensitive completion query failed:\n{:?}", err);
            receiver.await.map_err(|_e| err)
        }
    }
}

fn local_context(environment: &CompletionEnvironment) -> String {
    if let CompletionLocation::InlineData(ref inline_data) = environment.location {
        format!(
            "BIND({} AS ?qlue_ls_entity)",
            inline_data.visible_variables().first().unwrap().text(),
        )
    } else {
        panic!()
    }
}
