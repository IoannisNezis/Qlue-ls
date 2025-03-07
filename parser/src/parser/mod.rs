mod grammar;

use std::cell::Cell;

use crate::SyntaxKind;
use grammar::{parse_QueryUnit, parse_UpdateUnit};
use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    fuel: Cell<u32>,
    events: Vec<Event>,
}

#[derive(Debug, Clone)]
struct Token {
    kind: SyntaxKind,
    text: std::string::String,
}

impl Token {
    fn is_trivia(&self) -> bool {
        match self.kind {
            SyntaxKind::WHITESPACE => true,
            _ => false,
        }
    }
}

pub fn parse_text(input: &str, entry: TopEntryPoint) -> GreenNode {
    let tokens = lex(input);
    let parse_input = tokens
        .iter()
        .filter(|token| !token.is_trivia())
        .cloned()
        .collect();
    let output = entry.parse(parse_input);
    build_tree(tokens, output)
}

fn build_tree(tokens: Vec<Token>, events: Vec<Event>) -> GreenNode {
    let mut tokens = tokens.into_iter().peekable();
    let mut builder = GreenNodeBuilder::new();

    // Special case: pop the last `Close` event to ensure
    // that the stack is non-empty inside the loop.
    // assert!(matches!(events.pop(), Some(Event::Close)));
    for event in &events[..events.len() - 1] {
        match event {
            Event::Open { kind } => {
                while *kind != SyntaxKind::QueryUnit
                    && tokens.peek().map_or(false, |next| next.is_trivia())
                {
                    let token = tokens.next().unwrap();
                    builder.token(token.kind.into(), &token.text);
                }
                builder.start_node((*kind).into());
            }
            Event::Close => {
                builder.finish_node();
            }

            Event::Advance => {
                while tokens.peek().map_or(false, |next| next.is_trivia()) {
                    let token = tokens.next().unwrap();
                    builder.token(token.kind.into(), &token.text);
                }
                let token = tokens.next().unwrap();
                builder.token(token.kind.into(), &token.text);
            }
        }
    }
    // Eat trailing trivia tokens
    assert!(matches!(events.last(), Some(Event::Close)));
    while tokens.peek().map_or(false, |next| next.is_trivia()) {
        let token = tokens.next().unwrap();
        builder.token(token.kind.into(), &token.text);
    }
    builder.finish_node();
    builder.finish()
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

enum Event {
    Open { kind: SyntaxKind },
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
        // TODO: Error reporting.
        eprintln!("expected {kind:?}");
    }

    fn advance_with_error(&mut self, error: &str) {
        let m = self.open();
        // TODO: Error reporting.
        eprintln!("{error}");
        self.advance();
        self.close(m, SyntaxKind::Error);
    }
}

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

fn lex(text: &str) -> Vec<Token> {
    let mut lexer = SyntaxKind::lexer(text);
    let mut tokens = Vec::new();

    while let Some(result) = lexer.next() {
        tokens.push(Token {
            kind: result.unwrap_or(SyntaxKind::Error),
            text: lexer.slice().to_string(),
        });
    }
    return tokens;
}
