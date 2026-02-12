pub mod ast;
mod parser;
mod rules;
pub mod syntax_kind;
mod syntax_node;
mod utils;

pub use parser::{guess_operation_type, ParseError, TopEntryPoint};
use syntax_kind::SyntaxKind;
pub use syntax_node::*;
pub use utils::*;

use crate::parser::lex;

pub fn parse_query(input: &str) -> (SyntaxNode, Vec<ParseError>) {
    let (root, errors) = parser::parse_text(input, parser::TopEntryPoint::QueryUnit);
    (SyntaxNode::new_root(root), errors)
}

pub fn parse_update(input: &str) -> (SyntaxNode, Vec<ParseError>) {
    let (root, errors) = parser::parse_text(input, parser::TopEntryPoint::UpdateUnit);
    (SyntaxNode::new_root(root), errors)
}

pub fn parse(input: &str) -> (SyntaxNode, Vec<ParseError>) {
    match guess_operation_type(input) {
        Some(TopEntryPoint::QueryUnit) | None => parse_query(input),
        Some(TopEntryPoint::UpdateUnit) => parse_update(input),
    }
}

pub enum QueryType {
    SelectQuery,
    ConstructQuery,
    DescribeQuery,
    AskQuery,
}

pub fn guess_query_type(input: &str) -> Option<QueryType> {
    let tokens = lex(input);
    tokens.iter().find_map(|(token, _)| match token.kind() {
        SyntaxKind::SELECT => Some(QueryType::SelectQuery),
        SyntaxKind::CONSTRUCT => Some(QueryType::ConstructQuery),
        SyntaxKind::ASK => Some(QueryType::AskQuery),
        SyntaxKind::DESCRIBE => Some(QueryType::DescribeQuery),
        _ => None,
    })
}

#[cfg(test)]
mod tests;

