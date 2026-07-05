mod grammar;

use std::{cell::Cell, ops::Range};

use crate::SyntaxKind;
use grammar::{parse_QueryUnit, parse_UpdateUnit};
use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder, TextRange, TextSize};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    fuel: Cell<u32>,
    events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub(super) struct Token {
    kind: SyntaxKind,
    text: std::string::String,
}

impl Token {
    fn is_trivia(&self) -> bool {
        matches!(self.kind, SyntaxKind::WHITESPACE | SyntaxKind::Comment)
    }

    pub(super) fn kind(&self) -> SyntaxKind {
        self.kind
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub span: TextRange,
    pub message: String,
}

pub fn parse_text(input: &str, entry: TopEntryPoint) -> (GreenNode, Vec<ParseError>) {
    let tokens = lex(input);
    let parse_input = tokens
        .iter()
        .filter_map(|(token, _span)| (!token.is_trivia()).then_some(token))
        .cloned()
        .collect();
    let output = entry.parse(parse_input);
    build_tree(tokens, remove_empty_nodes(output))
}

/// Removes `Open`/`Close` pairs that contain no `Advance` event.
///
/// NOTE: The parser opens nodes speculatively during error recovery, which can
/// produce zero-width nodes. These break tree navigation (e.g. rowan's
/// `prev_token` gives up when a sibling subtree contains no tokens), so they
/// are dropped before the tree is built. The root node is always kept.
fn remove_empty_nodes(events: Vec<Event>) -> Vec<Event> {
    let mut remove = vec![false; events.len()];
    // INFO: stack of (index of `Open` event, node contains an `Advance`)
    let mut stack: Vec<(usize, bool)> = Vec::new();
    for (index, event) in events.iter().enumerate() {
        match event {
            Event::Open { .. } => stack.push((index, false)),
            Event::Advance => {
                if let Some(top) = stack.last_mut() {
                    top.1 = true;
                }
            }
            Event::Close => {
                let (open_index, has_tokens) = stack
                    .pop()
                    .expect("Every \"Close\" event should occur after a open event");
                if !has_tokens && !stack.is_empty() {
                    remove[open_index] = true;
                    remove[index] = true;
                } else if has_tokens {
                    if let Some(parent) = stack.last_mut() {
                        parent.1 = true;
                    }
                }
            }
            Event::Error { .. } => {}
        }
    }
    events
        .into_iter()
        .zip(remove)
        .filter_map(|(event, remove)| (!remove).then_some(event))
        .collect()
}

fn build_tree(
    tokens: Vec<(Token, Range<usize>)>,
    events: Vec<Event>,
) -> (GreenNode, Vec<ParseError>) {
    let mut cursor = 0;
    let mut builder = GreenNodeBuilder::new();
    let mut erros: Vec<ParseError> = Vec::new();

    // Special case: pop the last `Close` event to ensure
    // that the stack is non-empty inside the loop.
    // assert!(matches!(events.pop(), Some(Event::Close)));
    for event in &events[..events.len() - 1] {
        match event {
            Event::Open { kind } => {
                while !matches!(kind, SyntaxKind::QueryUnit | SyntaxKind::UpdateUnit)
                    && tokens
                        .get(cursor)
                        .map_or(false, |(next, _)| next.is_trivia())
                {
                    let (token, _) = &tokens[cursor];
                    builder.token(token.kind.into(), &token.text);
                    cursor += 1;
                }
                builder.start_node((*kind).into());
            }
            Event::Error { expected } => {
                // NOTE: the error is anchored to the next non-trivia token;
                // if there is none, to the end of the last non-trivia token
                let span = tokens[cursor..]
                    .iter()
                    .find(|(token, _)| !token.is_trivia())
                    .map(|(_, span)| {
                        TextRange::new(
                            TextSize::new(span.start as u32),
                            TextSize::new(span.end as u32),
                        )
                    })
                    .unwrap_or_else(|| {
                        let end = tokens[..cursor]
                            .iter()
                            .rev()
                            .find(|(token, _)| !token.is_trivia())
                            .map_or(0, |(_, span)| span.end as u32);
                        TextRange::new(TextSize::new(end), TextSize::new(end))
                    });
                erros.push(ParseError {
                    span,
                    message: if expected.is_empty() {
                        "Syntax Error: unexpected token".to_string()
                    } else {
                        format!(
                            "Syntax Error: expected {}",
                            expected
                                .into_iter()
                                .map(|kind| format!("{kind:?}"))
                                .collect::<Vec<_>>()
                                .join(" or ")
                        )
                    },
                });
            }
            Event::Close => {
                builder.finish_node();
            }

            Event::Advance => {
                while tokens
                    .get(cursor)
                    .map_or(false, |(next, _)| next.is_trivia())
                {
                    let (token, _) = &tokens[cursor];
                    builder.token(token.kind.into(), &token.text);
                    cursor += 1;
                }
                let (token, _) = &tokens[cursor];
                builder.token(token.kind.into(), &token.text);
                cursor += 1;
            }
        }
    }
    // Eat trailing trivia tokens
    assert!(matches!(events.last(), Some(Event::Close)));
    while tokens
        .get(cursor)
        .map_or(false, |(next, _)| next.is_trivia())
    {
        let (token, _) = &tokens[cursor];
        builder.token(token.kind.into(), &token.text);
        cursor += 1;
    }
    builder.finish_node();
    (builder.finish(), erros)
}

impl Parser {
    fn new(input: Vec<Token>) -> Self {
        Self {
            tokens: input,
            pos: 0,
            fuel: 1024.into(),
            events: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum Event {
    Open { kind: SyntaxKind },
    Error { expected: Vec<SyntaxKind> },
    Close,
    Advance,
}

// NOTE: Tokens that reliably mark the start or end of a major construct.
// Used by `advance_with_error` to synchronize after a syntax error.
const RECOVERY_TOKENS: &[SyntaxKind] = &[SyntaxKind::WHERE, SyntaxKind::LCurly, SyntaxKind::RCurly];

struct MarkOpened {
    index: usize,
}

impl Parser {
    fn open(&mut self) -> MarkOpened {
        let mark = MarkOpened {
            index: self.events.len(),
        };
        self.events.push(Event::Open {
            kind: SyntaxKind::Error,
        });
        mark
    }

    fn close(&mut self, m: MarkOpened, kind: SyntaxKind) {
        self.events[m.index] = Event::Open { kind };
        self.events.push(Event::Close);
    }

    fn advance(&mut self) {
        assert!(!self.eof());
        self.fuel.set(1024);
        self.events.push(Event::Advance);
        self.pos += 1;
    }

    fn eof(&self) -> bool {
        self.pos == self.tokens.len()
    }

    fn nth(&self, lookahead: usize) -> SyntaxKind {
        if self.fuel.get() == 0 {
            panic!("parser is stuck")
        }
        self.fuel.set(self.fuel.get() - 1);
        self.tokens
            .get(self.pos + lookahead)
            .map_or(SyntaxKind::Eof, |it| it.kind)
    }

    fn at(&self, kind: SyntaxKind) -> bool {
        self.nth(0) == kind
    }

    fn at_any(&self, kinds: &[SyntaxKind]) -> bool {
        let current = self.nth(0);
        kinds.contains(&current)
    }

    fn eat(&mut self, kind: SyntaxKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: SyntaxKind) {
        if self.eat(kind) {
            return;
        }
        self.events.push(Event::Error {
            expected: vec![kind],
        });
    }

    fn advance_with_error(&mut self, expected: Vec<SyntaxKind>) {
        self.events.push(Event::Error { expected });
        // NOTE: Recovery tokens are strong synchronization points. They are
        // never consumed as part of an error; instead the error is reported
        // and the token is left for an ancestor rule to consume.
        if self.at_any(RECOVERY_TOKENS) {
            return;
        }
        let m = self.open();
        self.advance();
        self.close(m, SyntaxKind::Error);
    }

    fn error_until(&mut self, expected: Vec<SyntaxKind>, recovery: &[SyntaxKind]) {
        self.events.push(Event::Error { expected }); // ONE diagnostic
        if self.at_any(recovery) || self.eof() {
            return; // nothing to skip
        }
        let m = self.open();
        while !self.at_any(recovery) && !self.eof() {
            self.advance(); // swallow all junk
        }
        self.close(m, SyntaxKind::Error); // ONE error node
    }

    pub(super) fn pos(&self) -> usize {
        self.pos
    }
}

#[derive(Debug)]
pub enum TopEntryPoint {
    QueryUnit,
    UpdateUnit,
}

impl TopEntryPoint {
    fn parse(&self, input: Vec<Token>) -> Vec<Event> {
        let mut parser = Parser::new(input);
        match self {
            TopEntryPoint::QueryUnit => parse_QueryUnit(&mut parser),
            TopEntryPoint::UpdateUnit => parse_UpdateUnit(&mut parser),
        }
        parser.events
    }
}

pub(super) fn lex(text: &str) -> Vec<(Token, Range<usize>)> {
    let mut lexer = SyntaxKind::lexer(text);
    let mut tokens = Vec::new();

    while let Some(result) = lexer.next() {
        tokens.push((
            Token {
                kind: result.unwrap_or(SyntaxKind::Error),
                text: lexer.slice().to_string(),
            },
            lexer.span(),
        ));
    }
    tokens
}

pub fn guess_operation_type(input: &str) -> Option<TopEntryPoint> {
    let tokens = lex(input);
    tokens.iter().find_map(|(token, _)| match token.kind {
        SyntaxKind::SELECT | SyntaxKind::CONSTRUCT | SyntaxKind::ASK | SyntaxKind::DESCRIBE => {
            Some(TopEntryPoint::QueryUnit)
        }
        SyntaxKind::LOAD
        | SyntaxKind::CLEAR
        | SyntaxKind::DROP
        | SyntaxKind::CREATE
        | SyntaxKind::ADD
        | SyntaxKind::MOVE
        | SyntaxKind::COPY
        | SyntaxKind::INSERT
        | SyntaxKind::INSERT_DATA
        | SyntaxKind::DELETE
        | SyntaxKind::DELETE_DATA
        | SyntaxKind::DELETE_WHERE => Some(TopEntryPoint::UpdateUnit),
        _ => None,
    })
}

#[cfg(test)]
mod tests;
