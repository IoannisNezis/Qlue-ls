use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::{
    ast::{AstNode, QueryUnit},
    parse_query, SyntaxNode,
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
    let cursor_offset = request
        .params
        .base
        .position
        .byte_index(&document.text)
        .ok_or(LSPError::new(
            ErrorCode::InvalidRequest,
            "given position is not inside document",
        ))?;
    let results = relevant_positions(
        document,
        root,
        request.params.previous.is_some_and(|prev| prev),
    );
    let jump_position = if request.params.previous.is_some_and(|prev| prev) {
        // NOTE: Jump to previous position
        let last = results.last().cloned();
        results
            .into_iter()
            .rev()
            .find(|(offset, _, _)| offset < &cursor_offset)
            .or(last)
    } else {
        // NOTE: Jump to next position
        let first = results.first().cloned();
        results
            .into_iter()
            .find(|(offset, _, _)| offset > &cursor_offset)
            .or(first)
    }
    .map(|(offset, before, after)| {
        JumpResult::new(
            Position::from_byte_index(offset.into(), &document.text).unwrap(),
            before,
            after,
        )
    });
    server.send_message(JumpResponse::new(request.get_id(), jump_position))?;
    Ok(())
}

fn relevant_positions(
    _document: &TextDocumentItem,
    root: SyntaxNode,
    jump_to_previous: bool,
) -> Vec<(TextSize, Option<&str>, Option<&str>)> {
    let mut res = Vec::new();
    if let Some(query_unit) = QueryUnit::cast(root) {
        // NOTE: End of select clause
        if let Some(offset) = query_unit.select_query().and_then(|sq| {
            if jump_to_previous {
                sq.where_clause().map(|wc| wc.syntax().text_range().start())
            } else {
                sq.select_clause().map(|sc| sc.syntax().text_range().end())
            }
        }) {
            res.push((
                offset,
                (!jump_to_previous).then_some(" "),
                jump_to_previous.then_some(" "),
            ));
        }
        // NOTE: End of group graph pattern
        if let Some((offset, has_children)) = query_unit
            .select_query()
            .and_then(|sq| sq.where_clause())
            .and_then(|sq| sq.group_graph_pattern())
            .and_then(|ggp| {
                ggp.syntax().last_child_or_token().map(|token| {
                    (
                        token.text_range().start(),
                        ggp.syntax().first_child().is_some(),
                    )
                })
            })
        {
            // FIXME: Here i assume that the tab_size is 2 and insert_spaces is true
            res.push((
                offset,
                has_children.then_some("  ").or(Some("\n  ")),
                Some("\n"),
            ));
        }
        // NOTE: End of soulution modifier
        if jump_to_previous {
            if let Some(offset) = query_unit
                .syntax()
                .last_token()
                .map(|token| token.text_range().end())
            {
                res.push((offset, None, None));
            }
        } else {
            if let Some(offset) = query_unit.select_query().and_then(|sq| {
                sq.soulution_modifier()
                    .map(|sm| sm.syntax().clone())
                    .or(sq.where_clause().map(|wc| wc.syntax().clone()))
                    .map(|node| node.text_range().end())
            }) {
                res.push((offset, Some("\n"), None));
            }
        }
    }
    res
}
