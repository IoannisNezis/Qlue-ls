use crate::server::lsp::{textdocument::TextDocumentItem, CodeAction};
use ll_sparql_parser::ast::Var;
mod add_aggregate_to_result;
mod add_to_result;
mod filter_var;
mod filter_var_lang;

pub(super) fn code_actions(var: Var, document: &TextDocumentItem) -> Vec<CodeAction> {
    let mut code_actions = Vec::new();
    if let Some(code_action) = add_to_result::code_action(&var, document) {
        code_actions.push(code_action)
    }
    if let Some(code_action) = filter_var_lang::code_action(&var, document) {
        code_actions.push(code_action)
    }
    if let Some(code_action) = filter_var::code_action(&var, document) {
        code_actions.push(code_action)
    }
    if let Some(code_action_vec) = add_aggregate_to_result::code_actions(&var, document) {
        for code_action in code_action_vec {
            code_actions.push(code_action)
        }
    }
    code_actions
}
