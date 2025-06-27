mod iri;
mod quickfix;
mod variable;
use crate::server::{
    lsp::{
        diagnostic::Diagnostic,
        errors::{ErrorCode, LSPError},
        CodeAction, CodeActionParams, CodeActionRequest, CodeActionResponse,
    },
    Server,
};
use futures::lock::Mutex;
use ll_sparql_parser::ast::{Iri, Var};
use ll_sparql_parser::{ast::AstNode, parse_query, TokenAtOffset};
use quickfix::get_quickfix;
use std::rc::Rc;

pub(crate) use quickfix::declare_prefix;
pub(crate) use quickfix::remove_prefix_declaration;

pub(super) async fn handle_codeaction_request(
    server_rc: Rc<Mutex<Server>>,
    request: CodeActionRequest,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
    let mut code_action_response = CodeActionResponse::new(request.get_id());
    code_action_response.add_code_actions(generate_code_actions(&mut server, &request.params)?);
    code_action_response.add_code_actions(generate_quickfixes(&mut server, request));
    server.send_message(code_action_response)
}

fn generate_quickfixes(server: &mut Server, request: CodeActionRequest) -> Vec<CodeAction> {
    request
        .params
        .context
        .diagnostics
        .into_iter()
        .filter_map(|diagnostic| {
            match get_quickfix(server, &request.params.text_document.uri, diagnostic) {
                Ok(code_action) => code_action,
                Err(err) => {
                    log::error!(
                        "Encountered Error while computing quickfix:\n{}\nDropping error!",
                        err.message
                    );
                    None
                }
            }
        })
        .collect()
}

fn generate_code_actions(
    server: &mut Server,
    params: &CodeActionParams,
) -> Result<Vec<CodeAction>, LSPError> {
    let document_uri = &params.text_document.uri;
    let document = server.state.get_document(document_uri)?;
    let root = parse_query(&document.text);
    let range = params
        .range
        .to_byte_index_range(&document.text)
        .ok_or(LSPError::new(
            ErrorCode::InvalidParams,
            &format!("Range ({:?}) not inside document range", params.range),
        ))?;

    if root.text_range().contains((range.end as u32).into()) {
        if let Some(token) = match root.token_at_offset((range.end as u32).into()) {
            TokenAtOffset::Single(token) | TokenAtOffset::Between(token, _) => Some(token),
            TokenAtOffset::None => None,
        } {
            let mut code_actions = vec![];
            if token
                .parent()
                .and_then(Iri::cast)
                .is_some_and(|iri| iri.is_uncompressed())
            {
                code_actions.extend(iri::code_actions(server, document.uri.clone()));
            } else if Var::can_cast(token.kind()) {
                code_actions.extend(variable::code_actions(&token, document))
            }

            return Ok(code_actions);
        }
    }
    Ok(vec![])
}
