use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::{
    SyntaxNode, SyntaxToken,
    ast::{AstNode, Triple},
    syntax_kind::SyntaxKind,
};
use text_size::TextSize;

use crate::server::{
    Server,
    lsp::{
        OnTypeFormattingRequest, OnTypeFormattingResponse,
        errors::{ErrorCode, LSPError},
        textdocument::{Position, Range, TextDocumentItem, TextEdit},
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
    let editor_position = request.get_position();
    let byte_offset_position: usize = editor_position
        .byte_index(&document.text)
        .ok_or(LSPError::new(
            ErrorCode::InvalidParams,
            "The requested position is not a valid position in the document.",
        ))?
        .into();

    let tree = server.state.get_cached_parse_tree(document_uri)?.tree;

    match request.get_char() {
        "\n" => handle_newline_trigger(
            &server,
            &request,
            document,
            tree,
            byte_offset_position,
            editor_position,
        ),
        ";" => handle_terminator_trigger(
            &server,
            &request,
            document,
            tree,
            byte_offset_position,
            editor_position,
            SyntaxKind::Semicolon,
        ),
        "." => handle_terminator_trigger(
            &server,
            &request,
            document,
            tree,
            byte_offset_position,
            editor_position,
            SyntaxKind::Dot,
        ),
        _ => Err(LSPError::new(
            ErrorCode::InvalidRequest,
            "Unsupported trigger character.",
        )),
    }
}

/// Handles the newline trigger character. This is the original on-type formatting behavior:
/// computes indentation based on brace nesting depth and predicate alignment.
fn handle_newline_trigger(
    server: &Server,
    request: &OnTypeFormattingRequest,
    document: &TextDocumentItem,
    tree: SyntaxNode,
    byte_offset_position: usize,
    editor_position: &Position,
) -> Result<(), LSPError> {
    // NOTE: Verify position is right after a newline.
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
        SyntaxKind::WHITESPACE,
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

/// Handles `;` and `.` trigger characters. When enabled via `auto_line_break` setting,
/// inserts a newline with correct indentation after a valid triple terminator.
fn handle_terminator_trigger(
    server: &Server,
    request: &OnTypeFormattingRequest,
    document: &TextDocumentItem,
    tree: SyntaxNode,
    byte_offset_position: usize,
    editor_position: &Position,
    expected_kind: SyntaxKind,
) -> Result<(), LSPError> {
    // NOTE: Check if auto_line_break is enabled. If not, return null response.
    if !server.settings.auto_line_break {
        return server.send_message(OnTypeFormattingResponse::null(request.get_id()));
    }

    // NOTE: Verify position is right after the expected terminator character.
    let expected_char = match expected_kind {
        SyntaxKind::Semicolon => ";",
        SyntaxKind::Dot => ".",
        _ => {
            return Err(LSPError::new(
                ErrorCode::InternalError,
                "Unexpected terminator kind.",
            ));
        }
    };
    if !(byte_offset_position > 0
        && document
            .text
            .get(byte_offset_position - 1..byte_offset_position)
            .is_some_and(|prev_char| prev_char == expected_char))
    {
        return Err(LSPError::new(
            ErrorCode::InvalidRequest,
            &format!(
                "Expected the trigger position to be right after '{}'.",
                expected_char
            ),
        ));
    }

    // NOTE: Find the terminator token (the character just typed).
    // The cursor is AFTER the terminator, so byte_offset_position - 1 is the START of the
    // terminator token. Use right_biased() to get the token starting at that offset.
    let trigger_token = tree
        .token_at_offset(TextSize::from((byte_offset_position) as u32))
        .left_biased()
        .ok_or(LSPError::new(
            ErrorCode::InternalError,
            "The requested position did not \"hit\" any syntax token.",
        ))?;

    // NOTE: Verify the token is indeed the expected terminator.
    if trigger_token.kind() != expected_kind {
        // INFO: The parser may not have placed the terminator token yet, or it's part of
        // another construct. Return null response.
        return server.send_message(OnTypeFormattingResponse::null(request.get_id()));
    }

    // NOTE: Find the Triple (TriplesSameSubjectPath) containing or preceding this terminator.
    let triple = find_triple_for_terminator(&trigger_token, expected_kind);

    // NOTE: Validate the triple is complete and has no parse errors.
    match triple {
        Some(t) if !t.has_error() => {}
        _ => {
            // INFO: Triple is invalid or not found. Return null response.
            return server.send_message(OnTypeFormattingResponse::null(request.get_id()));
        }
    };

    // NOTE: Compute indentation based on terminator type.
    let format_options = request.get_options();
    let tab_unit = if format_options.insert_spaces {
        " ".repeat(format_options.tab_size as usize)
    } else {
        "\t".to_string()
    };
    let base_indent_level = indent::brace_nesting_depth(&trigger_token);
    let base_indent = tab_unit.repeat(base_indent_level);

    let final_indent = match expected_kind {
        SyntaxKind::Semicolon => {
            // INFO: For semicolon, indent to predicate alignment or base + 1 tab.
            if server.settings.format.align_predicates {
                indent::predicate_alignment_column(&trigger_token, &document.text)
                    .map(|col| " ".repeat(col))
                    .unwrap_or_else(|| base_indent.clone() + &tab_unit)
            } else {
                base_indent + &tab_unit
            }
        }
        SyntaxKind::Dot => {
            // INFO: For dot, indent to the base brace nesting level (start of new triple).
            // NOTE: Check if the triple we're terminating is part of a contracted pattern.
            // If so, we need to check if there's a trailing dot already implied.
            // For now, use the brace nesting depth of the terminator token, which should
            // give us the correct indent for starting a new triple in the same block.
            base_indent
        }
        _ => base_indent,
    };

    // NOTE: Check if there's already whitespace after the terminator that we should skip/replace.
    // This handles the case where the user types ";" and we insert a newline.
    let existing_trailing: u32 = document.text[byte_offset_position..]
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .count() as u32;

    // NOTE: Insert newline + indentation at the current cursor position.
    let new_text = format!("\n{}", final_indent);
    let on_type_formatting_response = OnTypeFormattingResponse::new(
        request.get_id(),
        vec![TextEdit::new(
            Range::new(
                editor_position.line,
                editor_position.character,
                editor_position.line,
                editor_position.character + existing_trailing,
            ),
            &new_text,
        )],
    );
    server.send_message(on_type_formatting_response)
}

/// Finds the Triple (TriplesSameSubjectPath) associated with a terminator token.
///
/// For `;`: The semicolon is inside the PropertyListPath, so we walk up ancestors.
/// For `.`: The dot is a child of TriplesBlock, sibling to TriplesSameSubjectPath.
fn find_triple_for_terminator(token: &SyntaxToken, kind: SyntaxKind) -> Option<Triple> {
    match kind {
        SyntaxKind::Semicolon => {
            // NOTE: Semicolon is inside PropertyListPathNotEmpty, which is inside
            // TriplesSameSubjectPath. Walk up to find it.
            token
                .parent_ancestors()
                .find(|n| n.kind() == SyntaxKind::TriplesSameSubjectPath)
                .and_then(Triple::cast)
        }
        SyntaxKind::Dot => {
            // NOTE: Dot is a direct child of TriplesBlock. The TriplesSameSubjectPath
            // is a sibling (the first child of TriplesBlock).
            // Grammar: TriplesBlock = TriplesSameSubjectPath ( '.' TriplesBlock? )?
            token
                .parent()
                .filter(|p| p.kind() == SyntaxKind::TriplesBlock)
                .and_then(|block| block.children().find_map(Triple::cast))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ll_sparql_parser::parse_query;
    use text_size::TextSize;

    /// Get the token starting at a given offset (right_biased)
    fn token_at(input: &str, offset: usize) -> SyntaxToken {
        let (tree, _) = parse_query(input);
        tree.token_at_offset(TextSize::from(offset as u32))
            .right_biased()
            .expect("no token at offset")
    }

    #[test]
    fn test_find_triple_for_semicolon() {
        // "SELECT * WHERE { ?s ?p ?o; }"
        //  0         1         2
        //  0123456789012345678901234567
        //                          ^ semicolon starts at offset 25
        let input = "SELECT * WHERE { ?s ?p ?o; }";
        let semicolon = token_at(input, 25);
        assert_eq!(semicolon.kind(), SyntaxKind::Semicolon);

        let triple = find_triple_for_terminator(&semicolon, SyntaxKind::Semicolon);
        assert!(triple.is_some(), "Should find triple for semicolon");

        let triple = triple.unwrap();
        assert!(!triple.has_error(), "Triple should not have errors");
    }

    #[test]
    fn test_find_triple_for_dot() {
        // "SELECT * WHERE { ?s ?p ?o. }"
        //  0         1         2
        //  0123456789012345678901234567
        //                          ^ dot starts at offset 25
        let input = "SELECT * WHERE { ?s ?p ?o. }";
        let dot = token_at(input, 25);
        assert_eq!(dot.kind(), SyntaxKind::Dot);

        let triple = find_triple_for_terminator(&dot, SyntaxKind::Dot);
        assert!(triple.is_some(), "Should find triple for dot");

        let triple = triple.unwrap();
        assert!(!triple.has_error(), "Triple should not have errors");
    }

    #[test]
    fn test_invalid_triple_with_semicolon() {
        // "SELECT * WHERE { ?s ?p; }"
        //  0         1         2
        //  012345678901234567890123
        //                       ^ semicolon at offset 22, triple missing object
        let input = "SELECT * WHERE { ?s ?p; }";
        let semicolon = token_at(input, 22);
        assert_eq!(semicolon.kind(), SyntaxKind::Semicolon);

        let triple = find_triple_for_terminator(&semicolon, SyntaxKind::Semicolon);
        assert!(triple.is_some(), "Should find triple for semicolon");

        let triple = triple.unwrap();
        assert!(
            triple.has_error(),
            "Triple should have errors (missing object)"
        );
    }

    #[test]
    fn test_invalid_triple_with_dot() {
        // "SELECT * WHERE { ?s ?p. }"
        //  0         1         2
        //  012345678901234567890123
        //                       ^ dot at offset 22, triple missing object
        // NOTE: When the triple is invalid, the parser puts the dot inside an Error node,
        // so find_triple_for_terminator returns None. This is correct behavior - we don't
        // want to format invalid triples.
        let input = "SELECT * WHERE { ?s ?p. }";
        let dot = token_at(input, 22);
        assert_eq!(dot.kind(), SyntaxKind::Dot);

        let triple = find_triple_for_terminator(&dot, SyntaxKind::Dot);
        // For invalid triples, the dot is inside an Error node, so we can't find the triple.
        // This results in null response (no formatting), which is correct.
        assert!(
            triple.is_none(),
            "Should not find triple when dot is in Error node"
        );
    }
}
