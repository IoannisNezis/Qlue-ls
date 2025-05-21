use super::{
    environment,
    error::CompletionError,
    utils::{dispatch_completion_query, CompletionTemplate},
    variable, CompletionEnvironment,
};
use crate::server::{
    lsp::CompletionList, message_handler::completion::environment::CompletionLocation, Server,
};
use futures::lock::Mutex;
use ll_sparql_parser::ast::AstNode;
use std::rc::Rc;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let mut template_context = environment.template_context(server_rc.clone()).await;
    template_context.insert("local_context", &local_context(&environment));

    let mut completion_list = dispatch_completion_query(
        server_rc,
        &environment,
        template_context,
        CompletionTemplate::ObjectCompletion,
        false,
    )
    .await?;
    completion_list
        .items
        .extend(variable::completions_transformed(environment)?.items);
    Ok(completion_list)
}

fn local_context(environment: &CompletionEnvironment) -> Option<String> {
    if let CompletionLocation::Object(triple) = &environment.location {
        Some(format!(
            "{} {} ?qlue_ls_entity",
            triple.subject()?.text(),
            triple
                .properties_list_path()?
                .text_until(environment.anchor_token.as_ref()?.text_range().end())
        ))
    } else {
        None
    }
}
