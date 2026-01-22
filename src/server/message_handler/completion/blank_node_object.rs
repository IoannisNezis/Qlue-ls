use super::{
    environment::{CompletionEnvironment, CompletionLocation},
    error::CompletionError,
    utils::{CompletionTemplate, dispatch_completion_query},
};
use crate::server::{Server, lsp::CompletionList};
use futures::lock::Mutex;
use ll_sparql_parser::ast::AstNode;
use std::rc::Rc;

pub(super) async fn completions(
    server_rc: Rc<Mutex<Server>>,
    environment: &CompletionEnvironment,
) -> Result<CompletionList, CompletionError> {
    let mut template_context = environment.template_context().await;
    template_context.insert("local_context", &local_context(&environment));
    dispatch_completion_query(
        server_rc,
        &environment,
        template_context,
        CompletionTemplate::ObjectCompletionContextInsensitive,
        false,
    )
    .await
}

fn local_context(environment: &CompletionEnvironment) -> Option<String> {
    if let CompletionLocation::BlankNodeObject(ref blank_node_props) = environment.location {
        Some(format!(
            "[] {} ?qlue_ls_entity",
            blank_node_props.property_list()?.text(),
        ))
    } else {
        None
    }
}
