mod grammar;

use std::{cell::Cell, ops::Range};

use crate::SyntaxKind;
use grammar::{parse_QueryUnit, parse_UpdateUnit};
use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder, TextRange, TextSize};
use wasm_bindgen::prelude::wasm_bindgen;

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
    build_tree(tokens, output)
}

fn build_tree(
    tokens: Vec<(Token, Range<usize>)>,
    events: Vec<Event>,
) -> (GreenNode, Vec<ParseError>) {
    let mut tokens = tokens.into_iter().peekable();
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
                        .peek()
                        .map_or(false, |(next, _span)| next.is_trivia())
                {
                    let (token, _) = tokens.next().unwrap();
                    builder.token(token.kind.into(), &token.text);
                }
                builder.start_node((*kind).into());
            }
            Event::Error { expected } => {
                if let Some((_, span)) = tokens.peek() {
                    erros.push(ParseError {
                        span: TextRange::new(
                            TextSize::new(span.start as u32),
                            TextSize::new(span.start as u32),
                        ),
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
            }
            Event::Close => {
                builder.finish_node();
            }

            Event::Advance => {
                while tokens.peek().map_or(false, |(next, _)| next.is_trivia()) {
                    let (token, _) = tokens.next().unwrap();
                    builder.token(token.kind.into(), &token.text);
                }
                let (token, _) = tokens.next().unwrap();
                builder.token(token.kind.into(), &token.text);
            }
        }
    }
    // Eat trailing trivia tokens
    assert!(matches!(events.last(), Some(Event::Close)));
    while tokens.peek().map_or(false, |(next, _)| next.is_trivia()) {
        let (token, _) = tokens.next().unwrap();
        builder.token(token.kind.into(), &token.text);
    }
    builder.finish_node();
    (builder.finish(), erros)
}

impl Parser {
    fn new(input: Vec<Token>) -> Self {
        Self {
            tokens: input,
            pos: 0,
            fuel: 256.into(),
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
        self.fuel.set(256);
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
        kinds.iter().any(|kind| self.at(*kind))
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
        let m = self.open();
        self.advance();
        self.close(m, SyntaxKind::Error);
        self.events.push(Event::Error { expected });
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

#[wasm_bindgen]
pub fn determine_operation_type(input: &str) -> String {
    match guess_operation_type(input) {
        Some(TopEntryPoint::QueryUnit) => "Query",
        Some(TopEntryPoint::UpdateUnit) => "Update",
        None => "Unknown",
    }
    .to_string()
}

#[cfg(test)]
mod tests;
