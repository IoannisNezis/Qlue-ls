use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::{SyntaxElement, SyntaxNode};

use crate::server::{
    Server,
    lsp::{
        ParseTreeElement, ParseTreeRequest, ParseTreeResponse,
        errors::LSPError,
        textdocument::{Position, Range},
    },
};

pub(super) async fn handle_parse_tree_request(
    server_rc: Rc<Mutex<Server>>,
    request: ParseTreeRequest,
) -> Result<(), LSPError> {
    let server = server_rc.lock().await;
    let document_uri = &request.params.text_document.uri;
    let document = server.state.get_document(document_uri)?;
    let parse_result = server.state.get_cached_parse_tree(document_uri)?;
    let skip_trivia = request.params.skip_trivia.unwrap_or(false);
    let mut cursor = Cursor::new();
    let tree = build_element(&parse_result.tree, &document.text, skip_trivia, &mut cursor);
    server.send_message(ParseTreeResponse::new(
        &request.base.id,
        tree,
        parse_result.parse_time_ms,
    ))
}

/// Forward-only cursor that tracks the current byte offset and corresponding
/// LSP position. Advancing to a target byte is O(delta) instead of O(offset),
/// so building the full tree is O(n) total rather than O(n * m).
struct Cursor {
    byte_offset: usize,
    position: Position,
}

impl Cursor {
    fn new() -> Self {
        Self {
            byte_offset: 0,
            position: Position::new(0, 0),
        }
    }

    /// Advance the cursor to `target_byte`, updating the line/character position
    /// along the way. Returns the position at `target_byte`.
    fn advance_to(&mut self, target_byte: usize, text: &str) -> Position {
        for chr in text[self.byte_offset..target_byte].chars() {
            match chr {
                '\n' => {
                    self.position.line += 1;
                    self.position.character = 0;
                }
                _ => {
                    self.position.character += chr.len_utf16() as u32;
                }
            }
            self.byte_offset += chr.len_utf8();
        }
        self.position
    }
}

fn build_element(
    node: &SyntaxNode,
    text: &str,
    skip_trivia: bool,
    cursor: &mut Cursor,
) -> ParseTreeElement {
    let node_range = node.text_range();
    let start = cursor.advance_to(node_range.start().into(), text);
    let children: Vec<ParseTreeElement> = node
        .children_with_tokens()
        .filter(|child| !skip_trivia || !child.kind().is_trivia())
        .map(|child| match child {
            SyntaxElement::Node(child_node) => {
                build_element(&child_node, text, skip_trivia, cursor)
            }
            SyntaxElement::Token(token) => {
                let token_range = token.text_range();
                let token_start = cursor.advance_to(token_range.start().into(), text);
                let token_end = cursor.advance_to(token_range.end().into(), text);
                ParseTreeElement::Token {
                    kind: format!("{:?}", token.kind()),
                    range: Range {
                        start: token_start,
                        end: token_end,
                    },
                    text: format!("{:?}", token.text().to_string()),
                }
            }
        })
        .collect();
    // NOTE: when skip_trivia is on, trailing trivia within the node may not have
    // been visited by the children loop, so we advance to the node end here.
    let end = cursor.advance_to(node_range.end().into(), text);
    ParseTreeElement::Node {
        kind: format!("{:?}", node.kind()),
        range: Range { start, end },
        children,
    }
}
