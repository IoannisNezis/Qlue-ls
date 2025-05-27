use std::rc::Rc;

use super::{
    environment::{CompletionEnvironment, CompletionLocation},
    error::CompletionError,
    utils::{dispatch_completion_query, CompletionTemplate},
};
use crate::server::{lsp::CompletionList, message_handler::completion::utils::reduce_path, Server};
use futures::lock::Mutex;
use ll_sparql_parser::syntax_kind::SyntaxKind;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let mut template_context = environment.template_context().await;
    template_context.insert("local_context", &local_context(&environment));

    dispatch_completion_query(
        server_rc,
        &environment,
        template_context,
        CompletionTemplate::PredicateCompletion,
        true,
    )
    .await
}

fn local_context(environment: &CompletionEnvironment) -> Option<String> {
    if let CompletionLocation::BlankNodeProperty(ref prop_list) = environment.location {
        if environment
            .continuations
            .contains(&SyntaxKind::PropertyListPath)
            || environment
                .continuations
                .contains(&SyntaxKind::PropertyListPathNotEmpty)
            || prop_list.property_list().is_none()
        {
            Some("[] ?qlue_ls_entity []".to_string())
        } else {
            let properties = prop_list.property_list().unwrap().properties();
            if environment.continuations.contains(&SyntaxKind::VerbPath) {
                Some("[] ?qlue_ls_entity []".to_string())
            } else if properties.len() == 1 {
                reduce_path(
                    "[]",
                    &properties[0].verb,
                    "[]",
                    environment
                        .anchor_token
                        .as_ref()
                        .unwrap()
                        .text_range()
                        .end(),
                )
            } else {
                let (last_prop, prev_prop) = properties.split_last()?;
                Some(format!(
                    "[] {} . {}",
                    prev_prop
                        .iter()
                        .map(|prop| prop.text())
                        .collect::<Vec<_>>()
                        .join(" ; "),
                    reduce_path(
                        "[]",
                        &last_prop.verb,
                        "[]",
                        environment
                            .anchor_token
                            .as_ref()
                            .unwrap()
                            .text_range()
                            .end()
                    )?
                ))
            }
        }
    } else {
        None
    }
}
