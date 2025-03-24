mod add_to_result;
mod filter_var;
mod quickfix;
use std::collections::HashSet;

use ll_sparql_parser::{parse_query, syntax_kind::SyntaxKind, TokenAtOffset};
use quickfix::get_quickfix;

use crate::server::{
    anaysis::{get_all_uncompacted_uris, get_declared_uri_prefixes},
    lsp::{
        diagnostic::{Diagnostic, DiagnosticCode},
        errors::{ErrorCode, LSPError},
        textdocument::{Range, TextDocumentItem, TextEdit},
        CodeAction, CodeActionKind, CodeActionParams, CodeActionRequest, CodeActionResponse,
    },
    Server,
};

pub(super) async fn handle_codeaction_request(
    server: &mut Server,
    request: CodeActionRequest,
) -> Result<(), LSPError> {
    let mut code_action_response = CodeActionResponse::new(request.get_id());
    code_action_response.add_code_actions(generate_code_actions(server, &request.params)?);
    code_action_response.add_code_actions(
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
            .collect(),
    );
    server.send_message(code_action_response)
}

fn generate_code_actions(
    server: &Server,
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
                    .map_or(false, |parent| parent.kind() == SyntaxKind::iri)
            {
                if let Some(code_action) = shorten_all_uris(server, document) {
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
fn shorten_all_uris(server: &Server, document: &TextDocumentItem) -> Option<CodeAction> {
    let mut code_action = CodeAction::new("Shorten all URI's", Some(CodeActionKind::Refactor));
    let uncompacted_uris = get_all_uncompacted_uris(server, &document.uri).ok()?;
    let mut declared_uri_prefix_set: HashSet<String> =
        get_declared_uri_prefixes(&server.state, &document.uri)
            .ok()?
            .into_iter()
            .map(|(uri, _range)| uri[1..uri.len() - 1].to_string())
            .collect();

    uncompacted_uris.iter().for_each(|(uri, range)| {
        if let Some((prefix, uri_prefix, curie)) = server.shorten_uri(&uri[1..uri.len() - 1]) {
            code_action.add_edit(&document.uri, TextEdit::new(range.clone(), &curie));
            if !declared_uri_prefix_set.contains(&uri_prefix) {
                code_action.add_edit(
                    &document.uri,
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
    use indoc::indoc;
    use tree_sitter::Parser;
    use tree_sitter_sparql::LANGUAGE;

    use crate::server::{
        lsp::textdocument::{Range, TextDocumentItem, TextEdit},
        message_handler::code_action::shorten_all_uris,
        state::ServerState,
        Server,
    };

    fn setup_state(text: &str) -> ServerState {
        let mut state = ServerState::new();
        let mut parser = Parser::new();
        if let Err(err) = parser.set_language(&LANGUAGE.into()) {
            log::error!("Could not initialize parser:\n{}", err)
        }
        let document = TextDocumentItem::new("uri", text);
        let tree = parser.parse(&document.text, None);
        state.add_document(document, tree);
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
        let document = server.state.get_document("uri").unwrap();
        let code_action = shorten_all_uris(&server, document).unwrap();
        assert_eq!(
            code_action.edit.changes.get("uri").unwrap(),
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
        let document = server.state.get_document("uri").unwrap();
        let code_action = shorten_all_uris(&server, document).unwrap();
        assert_eq!(
            code_action.edit.changes.get("uri").unwrap(),
            &vec![
                TextEdit::new(Range::new(2, 5, 2, 30), "schema:name"),
                TextEdit::new(Range::new(3, 5, 3, 30), "schema:name"),
            ]
        );
    }
}
