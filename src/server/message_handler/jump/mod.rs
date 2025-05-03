use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::{
    ast::{AstNode, QueryUnit},
    parse_query,
    syntax_kind::SyntaxKind,
    SyntaxNode, TokenAtOffset,
};
use text_size::TextSize;

use crate::server::{
    lsp::{
        errors::{ErrorCode, LSPError},
        textdocument::{Position, TextDocumentItem},
        JumpRequest, JumpResponse, JumpResult,
    },
    Server,
};

pub(super) async fn handle_jump_request(
    server_rc: Rc<Mutex<Server>>,
    request: JumpRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let document_uri = &request.params.base.text_document.uri;
    let document = server.state.get_document(document_uri)?;
    let root = parse_query(&document.text);
    let offset = request
        .params
        .base
        .position
        .to_byte_index(&document.text)
        .ok_or(LSPError::new(
            ErrorCode::InvalidRequest,
            "given position is not inside document",
        ))?;
    let current_offset = match root.token_at_offset(TextSize::new(offset as u32)) {
        TokenAtOffset::Single(mut token) | TokenAtOffset::Between(mut token, _) => {
            while let Some(next) = token.next_token() {
                if matches!(next.kind(), SyntaxKind::WHITESPACE) {
                    token = next;
                } else {
                    break;
                }
            }
            token.text_range().end()
        }
        TokenAtOffset::None => TextSize::new(offset as u32),
    };
    let results = relevant_positions(document, root);
    let first = results.first().cloned();
    let next = results
        .into_iter()
        .find(|(offset, _, _)| offset > &current_offset)
        .or(first)
        .map(|(offset, before, after)| {
            JumpResult::new(
                Position::from_byte_index(offset.into(), &document.text).unwrap(),
                before,
                after,
            )
        });
    server.send_message(JumpResponse::new(request.get_id(), next))?;
    Ok(())
}

fn relevant_positions(
    document: &TextDocumentItem,
    root: SyntaxNode,
) -> Vec<(TextSize, Option<&str>, Option<&str>)> {
    let mut res = Vec::new();
    if let Some(query_unit) = QueryUnit::cast(root) {
        if let Some(offset) = query_unit
            .select_query()
            .and_then(|sq| sq.select_clause())
            .map(|sc| sc.syntax().text_range().end())
        {
            res.push((offset, Some(" "), None));
        }
        if let Some(offset) = query_unit
            .select_query()
            .and_then(|sq| sq.where_clause())
            .and_then(|sq| sq.group_graph_pattern())
            .and_then(|ggp| ggp.syntax().last_child_or_token())
            .map(|child| child.text_range().start())
        {
            res.push((offset, Some("\n  "), Some("\n")));
        }
        if let Some(offset) = query_unit
            .select_query()
            .and_then(|sq| sq.where_clause())
            .map(|wc| wc.syntax().text_range().end())
        {
            res.push((offset, Some("\n"), None));
        }
    }
    res
}
