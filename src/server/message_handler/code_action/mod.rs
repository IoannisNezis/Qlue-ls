mod add_to_result;
mod filter_var;
mod quickfix;
use std::{collections::HashSet, rc::Rc};

use futures::lock::Mutex;
use ll_sparql_parser::{ast::AstNode, parse_query, syntax_kind::SyntaxKind, TokenAtOffset};
use quickfix::get_quickfix;

pub(crate) use quickfix::declare_prefix;
pub(crate) use quickfix::remove_prefix_declaration;

use crate::server::{
    anaysis::{find_all_prefix_declarations, find_all_uncompacted_iris},
    lsp::{
        diagnostic::Diagnostic,
        errors::{ErrorCode, LSPError},
        textdocument::{Range, TextEdit},
        CodeAction, CodeActionKind, CodeActionParams, CodeActionRequest, CodeActionResponse,
    },
    Server,
};

pub(super) async fn handle_codeaction_request(
    server_rc: Rc<Mutex<Server>>,
    request: CodeActionRequest,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
    let mut code_action_response = CodeActionResponse::new(request.get_id());
    code_action_response.add_code_actions(generate_code_actions(&mut server, &request.params)?);
    code_action_response.add_code_actions(
        request
            .params
            .context
            .diagnostics
            .into_iter()
            .filter_map(|diagnostic| {
                match get_quickfix(&mut server, &request.params.text_document.uri, diagnostic) {
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
            .collect(),
    );
    server.send_message(code_action_response)
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
            if token.kind() == SyntaxKind::IRIREF
                && token
                    .parent()
                    .is_some_and(|parent| parent.kind() == SyntaxKind::iri)
            {
                if let Some(code_action) = shorten_all_uris(server, document_uri) {
                    code_actions.push(code_action)
                }
            } else if [SyntaxKind::VAR1, SyntaxKind::VAR2].contains(&token.kind()) {
                if let Some(code_action) = add_to_result::code_action(&token, document) {
                    code_actions.push(code_action)
                }
                if let Some(code_action) = filter_var::code_action(&token, document) {
                    code_actions.push(code_action)
                }
            }

            return Ok(code_actions);
        }
    }
    Ok(vec![])
}

// TODO: Handle errors properly.
fn shorten_all_uris(server: &mut Server, document_uri: &String) -> Option<CodeAction> {
    let mut code_action = CodeAction::new("Shorten all URI's", Some(CodeActionKind::Refactor));
    let uncompacted_uris = find_all_uncompacted_iris(server, document_uri).ok()?;
    let mut declared_uri_prefix_set: HashSet<String> =
        find_all_prefix_declarations(&mut server.state, document_uri)
            .ok()?
            .into_iter()
            .filter_map(|prefix_declaration| prefix_declaration.raw_uri_prefix())
            .collect();
    let document = server.state.get_document(document_uri).ok()?;
    uncompacted_uris.iter().for_each(|iri| {
        if let Some((prefix, uri_prefix, curie)) =
            server.shorten_uri(&iri.raw_iri().expect("iri should be uncompacted"), None)
        {
            code_action.add_edit(
                document_uri,
                TextEdit::new(
                    Range::from_byte_offset_range(iri.syntax().text_range(), &document.text)
                        .unwrap(),
                    &curie,
                ),
            );
            if !declared_uri_prefix_set.contains(&uri_prefix) {
                code_action.add_edit(
                    document_uri,
                    TextEdit::new(
                        Range::new(0, 0, 0, 0),
                        &format!("PREFIX {}: <{}>\n", prefix, uri_prefix),
                    ),
                );
                declared_uri_prefix_set.insert(uri_prefix);
            }
        }
    });
    if !uncompacted_uris.is_empty() {
        return Some(code_action);
    }

    None
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::server::{
        lsp::{
            textdocument::{Range, TextDocumentItem, TextEdit},
            Backend,
        },
        message_handler::code_action::shorten_all_uris,
        state::ServerState,
        Server,
    };
    use indoc::indoc;

    fn setup_state(text: &str) -> ServerState {
        let mut state = ServerState::new();
        state.add_backend(Backend {
            name: "test".to_string(),
            url: "".to_string(),
            health_check_url: None,
        });
        state.set_default_backend("test".to_string());
        state
            .add_prefix_map_test(
                "test".to_string(),
                HashMap::from_iter([("schema".to_string(), "https://schema.org/".to_string())]),
            )
            .unwrap();
        let document = TextDocumentItem::new("uri", text);
        state.add_document(document);
        state
    }

    #[test]
    fn shorten_all_uris_undeclared() {
        let mut server = Server::new(|_message| {});
        let state = setup_state(indoc!(
            "SELECT * {
               ?a <https://schema.org/name> ?b .
               ?c <https://schema.org/name> ?d
             }"
        ));
        server.state = state;
        let code_action = shorten_all_uris(&mut server, &"uri".to_string()).unwrap();
        assert_eq!(
            code_action.edit.changes.unwrap().get("uri").unwrap(),
            &vec![
                TextEdit::new(Range::new(1, 5, 1, 30), "schema:name"),
                TextEdit::new(
                    Range::new(0, 0, 0, 0),
                    "PREFIX schema: <https://schema.org/>\n"
                ),
                TextEdit::new(Range::new(2, 5, 2, 30), "schema:name"),
            ]
        );
    }

    #[test]
    fn shorten_all_uris_declared() {
        let mut server = Server::new(|_message| {});
        let state = setup_state(indoc!(
            "PREFIX schema: <https://schema.org/>
             SELECT * {
               ?a <https://schema.org/name> ?b .
               ?c <https://schema.org/name> ?d
             }"
        ));
        server.state = state;
        let code_action = shorten_all_uris(&mut server, &"uri".to_string()).unwrap();
        assert_eq!(
            code_action.edit.changes.unwrap().get("uri").unwrap(),
            &vec![
                TextEdit::new(Range::new(2, 5, 2, 30), "schema:name"),
                TextEdit::new(Range::new(3, 5, 3, 30), "schema:name"),
            ]
        );
    }
}
