use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::syntax_kind::SyntaxKind;
use text_size::TextSize;

use crate::server::{
    Server,
    lsp::{
        OnTypeFormattingRequest, OnTypeFormattingResponse,
        errors::{ErrorCode, LSPError},
        textdocument::{Range, TextEdit},
    },
    message_handler::indent,
};

pub(super) async fn handle_on_type_format_request(
    server_rc: Rc<Mutex<Server>>,
    request: OnTypeFormattingRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let document_uri = request.get_document_uri();
    let document = server.state.get_document(document_uri)?;
    let editor_position = request.get_position().clone();
    let byte_offset_position: usize = editor_position
        .byte_index(&document.text)
        .ok_or(LSPError::new(
            ErrorCode::InvalidParams,
            "The requested position is not a valid position in the document.",
        ))?
        .into();
    // NOTE: Veryfy trigger char and position.
    // The char should be "\n" and the position should be right after a newline.
    if request.get_char() != "\n" {
        return Err(LSPError::new(
            ErrorCode::InvalidRequest,
            "Expected the trigger character to be \"\\n\".",
        ));
    }
    if !(byte_offset_position > 0
        && document
            .text
            .get(byte_offset_position - 1..byte_offset_position)
            .is_some_and(|prev_char| prev_char == "\n"))
    {
        return Err(LSPError::new(
            ErrorCode::InvalidRequest,
            "Expected the trigger position to be right after a newline.",
        ));
    }

    // NOTE: compute nesting depth at the trigger position.
    let tree = server.state.get_cached_parse_tree(document_uri)?.tree;
    let trigger_token = tree
        .token_at_offset(TextSize::from(byte_offset_position as u32))
        .left_biased()
        .ok_or(LSPError::new(
            ErrorCode::InternalError,
            "The requested position did not \"hit\" any syntax token.",
        ))?;
    let indentation_level = indent::brace_nesting_depth(&trigger_token);

    // NOTE: compute base indent from the client-supplied FormattingOptions.
    let format_options = request.get_options();
    let tab_unit = if format_options.insert_spaces {
        " ".repeat(format_options.tab_size as usize)
    } else {
        "\t".to_string()
    };
    let indent = tab_unit.repeat(indentation_level);

    // WARNING: The trigger token at byte_offset_position should always be WHITESPACE.
    assert_eq!(
        trigger_token.kind(),
        ll_sparql_parser::syntax_kind::SyntaxKind::WHITESPACE,
        "trigger token at byte offset {} should be WHITESPACE, got {:?}",
        byte_offset_position,
        trigger_token.kind()
    );
    // NOTE: count existing whitespace after the trigger position so the edit replaces it
    // rather than prepending to it. Spaces and tabs are ASCII, so char count == UTF-16 count.
    let existing_whitespace: u32 = document.text[byte_offset_position..]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .count() as u32;

    // NOTE: Walk backwards through tokens, skipping trivia, to find the last meaningful token
    // before the newline. If it is a Semicolon, the new line needs predicate-continuation indent.
    // NOTE: The semicolon token itself is passed to predicate_alignment_column because the
    // trigger (whitespace) token lives outside the TriplesSameSubjectPath in the parse tree —
    // the semicolon is always inside it.
    let prev_semicolon = std::iter::successors(trigger_token.prev_token(), |t| t.prev_token())
        .find(|t| !t.kind().is_trivia())
        .filter(|t| t.kind() == SyntaxKind::Semicolon);

    let final_indent = if let Some(ref semicolon) = prev_semicolon {
        if server.settings.format.align_predicates {
            // Align with the column of the first predicate in this triple.
            // Falls back to indent + tab_unit if the triple ancestor cannot be found.
            let col = indent::predicate_alignment_column(semicolon, &document.text);
            match col {
                Some(col) => " ".repeat(col),
                None => indent + &tab_unit,
            }
        } else {
            // One tab unit deeper than the brace-depth indent.
            indent + &tab_unit
        }
    } else {
        indent
    };

    let on_type_formatting_response = OnTypeFormattingResponse::new(
        request.get_id(),
        vec![TextEdit::new(
            Range::new(
                editor_position.line,
                editor_position.character,
                editor_position.line,
                editor_position.character + existing_whitespace,
            ),
            &final_indent,
        )],
    );
    server.send_message(on_type_formatting_response)
}
