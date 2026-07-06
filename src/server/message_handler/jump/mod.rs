use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::{
    SyntaxNode,
    ast::{AstNode, GroupGraphPattern, QueryUnit},
    parse,
    syntax_kind::SyntaxKind,
};
use text_size::{TextRange, TextSize};

use crate::server::{
    Server,
    lsp::{
        FormattingOptions, JumpRequest, JumpResponse, JumpResult,
        errors::{ErrorCode, LSPError},
        textdocument::{Position, Range, TextEdit},
    },
    message_handler::formatting::format_document,
};

#[tracing::instrument(skip_all, fields(id = %request.get_id(), uri = %request.params.base.text_document.uri))]
pub(super) async fn handle_jump_request(
    server_rc: Rc<Mutex<Server>>,
    request: JumpRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let document_uri = &request.params.base.text_document.uri;
    let document = server.state.get_document(document_uri)?;
    let jump_to_prev = request.params.previous.is_some_and(|prev| prev);
    let root = server.state.get_cached_parse_tree(document_uri)?.tree;
    let cursor_offset = request
        .params
        .base
        .position
        .byte_index(&document.text)
        .ok_or(LSPError::new(
            ErrorCode::InvalidRequest,
            "given position is not inside document",
        ))?;
    let default_options = FormattingOptions {
        tab_size: 2,
        insert_spaces: true,
    };
    let options = request.params.options.as_ref().unwrap_or(&default_options);
    // NOTE: Format the document first, so the jump target is computed on the
    // formatted document and the format edits cannot invalidate the cursor.
    // Contraction is disabled since it produces a whole-document edit,
    // which makes tracking the cursor through the edits impossible.
    let mut format_settings = server.settings.format.clone();
    format_settings.contract_triples = false;
    let format_edits = format_document(document, root, options, &format_settings)?;
    let formatted_cursor_offset =
        map_offset_through_edits(cursor_offset, &format_edits, &document.text);
    let mut formatted_document = document.clone();
    formatted_document.apply_text_edits(format_edits);
    let (formatted_root, _) = parse(&formatted_document.text);

    let results = relevant_positions(formatted_root, jump_to_prev);
    let target = if jump_to_prev {
        // NOTE: Jump to previous position
        let last = results.last().cloned();
        results
            .into_iter()
            .rev()
            .find(|(offset, _, _)| offset < &formatted_cursor_offset)
            .or(last)
    } else {
        // NOTE: Jump to next position
        let first = results.first().cloned();
        results
            .into_iter()
            .find(|(offset, _, _)| offset > &formatted_cursor_offset)
            .or(first)
    };

    // NOTE: Insert the placeholder text around the jump target and compute the
    // final cursor position in the resulting document.
    let mut final_text = formatted_document.text;
    let position = target.and_then(|(offset, before, after)| {
        let before = before.unwrap_or("");
        let after = after.unwrap_or("");
        final_text.insert_str(offset.into(), &format!("{before}{after}"));
        Position::from_byte_index(offset + TextSize::of(before), &final_text)
    });

    let result = JumpResult {
        edits: vec![TextEdit::new(
            Range::new(0, 0, u32::MAX, u32::MAX),
            &final_text,
        )],
        position,
    };
    server.send_message(JumpResponse::new(request.get_id(), Some(result)))?;
    Ok(())
}

/// Map a byte offset in the original document to the corresponding byte offset
/// after the given `TextEdit`s are applied.
///
/// If the offset lies inside a replaced range, it is clamped to the end of the
/// replacement text.
fn map_offset_through_edits(offset: TextSize, edits: &[TextEdit], text: &str) -> TextSize {
    let mut byte_edits: Vec<(TextRange, i64)> = edits
        .iter()
        .filter_map(|edit| {
            edit.range
                .to_byte_index_range(text)
                .map(|range| (range, edit.new_text.len() as i64))
        })
        .collect();
    byte_edits.sort_by_key(|(range, _)| range.start());
    let mut delta: i64 = 0;
    for (range, new_length) in byte_edits {
        if range.end() <= offset {
            delta += new_length - u32::from(range.len()) as i64;
        } else if range.start() < offset {
            return TextSize::new((u32::from(range.start()) as i64 + delta + new_length) as u32);
        } else {
            break;
        }
    }
    TextSize::new((u32::from(offset) as i64 + delta) as u32)
}

fn relevant_positions(
    root: SyntaxNode,
    jump_to_previous: bool,
) -> Vec<(TextSize, Option<&'static str>, Option<&'static str>)> {
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

        for (offset, has_children) in query_unit
            .syntax()
            .descendants()
            .filter_map(GroupGraphPattern::cast)
            .filter_map(|ggp| {
                ggp.syntax().last_child_or_token().map(|token| {
                    (
                        token.text_range().start(),
                        ggp.syntax().first_child().is_some(),
                    )
                })
            })
        {
            res.push((
                offset,
                has_children.then_some("  ").or(Some("\n  ")),
                Some("\n"),
            ));
        }
        // NOTE: End of soulution modifier OR end of query if no solution modifier is present
        let mut last_token = query_unit.syntax().last_token();
        while last_token
            .as_ref()
            .is_some_and(|token| token.kind() == SyntaxKind::WHITESPACE)
        {
            last_token = last_token.and_then(|token| token.prev_token());
        }
        if let Some(offset) = last_token.map(|token| token.text_range().end()) {
            res.push((offset, Some("\n"), None));
        }
    }
    res.sort_by(|a, b| a.0.cmp(&b.0));
    res
}
