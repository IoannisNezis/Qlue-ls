use std::fmt::Display;

use ll_sparql_parser::{
    ast::{AstNode, GroupGraphPattern},
    syntax_kind::SyntaxKind,
    SyntaxNode, SyntaxToken,
};

use super::CompletionLocation;

#[derive(Debug)]
pub(crate) struct Context {
    pub nodes: Vec<SyntaxNode>,
    pub prefixes: Vec<String>,
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .nodes
            .iter()
            .map(|node| node.to_string())
            .collect::<Vec<_>>()
            .join(" .\n");
        write!(f, "{}", s)
    }
}

pub(super) fn context(
    location: &CompletionLocation,
    maybe_anchor: Option<&SyntaxToken>,
) -> Option<Context> {
    let anchor = maybe_anchor?;
    match location {
        CompletionLocation::Predicate(triple) | CompletionLocation::Object(triple) => {
            let group_graph_pattern = triple
                .triples_block()
                .and_then(|triples_block| triples_block.group_graph_pattern())?;
            let (nodes, prefixes) = context_down_tree(&group_graph_pattern);
            Some(Context { nodes, prefixes })
        }
        _ => None,
    }
}

fn context_down_tree(group_graph_pattern: &GroupGraphPattern) -> (Vec<SyntaxNode>, Vec<String>) {
    let triples: Vec<_> = group_graph_pattern
        .triple_blocks()
        .into_iter()
        .flat_map(|triples_block| triples_block.triples())
        .collect();
    let not_triples: Vec<_> = group_graph_pattern
        .group_pattern_not_triples()
        .into_iter()
        .filter(|not_triples| not_triples.syntax().kind() != SyntaxKind::OptionalGraphPattern)
        .collect();
    let prefixes = triples
        .iter()
        .flat_map(|triple| triple.used_prefixes())
        .chain(not_triples.iter().flat_map(|nt| nt.used_prefixes()))
        .collect();
    (
        triples
            .iter()
            .filter_map(|triple| (!triple.has_error()).then_some(triple.syntax().clone()))
            .chain(not_triples.iter().map(|nt| nt.syntax().clone()))
            .collect(),
        prefixes,
    )
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use ll_sparql_parser::parse_query;

    use crate::server::{
        lsp::textdocument::Position,
        message_handler::completion::environment::{
            get_anchor_token, get_continuations, get_location, get_trigger_token,
        },
    };

    use super::{context, Context};

    fn location_at(input: &str, cursor: Position) -> Context {
        let root = parse_query(input);
        let offset = (cursor.to_byte_index(input).unwrap() as u32).into();
        let trigger_token = get_trigger_token(&root, offset).unwrap();
        let anchor = get_anchor_token(trigger_token);
        let continuations = get_continuations(&root, &anchor);
        let location = get_location(&anchor, &continuations, offset);
        context(&location, anchor.as_ref()).unwrap()
    }

    #[test]
    fn context_inner_block() {
        let input = indoc! {
            "Select * {
                ?s <p1> <o1> .
                ?s <p2> <o2> .
                ?s 
             }
            "
        };
        let position = Position::new(3, 6);
        let context = location_at(input, position);
        assert_eq!(
            context.to_string(),
            indoc! {
                "?s <p1> <o1> .
                 ?s <p2> <o2>"
            }
        );
    }

    #[test]
    fn context_inter_block() {
        let input = indoc! {
            "Select * {
                ?s <p1> <o1>
                FILTER (?s)
                ?s <p2> <o2> .
                ?s 
             }
            "
        };
        let position = Position::new(4, 6);
        let context = location_at(input, position);
        assert_eq!(
            context.to_string(),
            indoc! {
                "?s <p1> <o1> .
                 ?s <p2> <o2> .
                 FILTER (?s)"
            }
        );
    }

    #[test]
    fn context_super_block() {
        let input = indoc! {
            "Select * {
                ?s <p1> <o1>
                {
                  ?s 
                }
             }
            "
        };
        let position = Position::new(3, 8);
        let context = location_at(input, position);
        assert_eq!(
            context.to_string(),
            indoc! {
                "?s <p1> <o1>"
            }
        );
    }
}
