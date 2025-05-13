use std::fmt::Display;

use ll_sparql_parser::{ast::AstNode, SyntaxNode, SyntaxToken};

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
            let triples: Vec<_> = triple
                .triples_block()
                .and_then(|triples_block| triples_block.group_graph_pattern())
                .map(|ggp| {
                    ggp.triple_blocks()
                        .into_iter()
                        .flat_map(|triples_block| triples_block.triples())
                })?
                .into_iter()
                .filter(|triple| {
                    !triple
                        .syntax()
                        .text_range()
                        .contains_range(anchor.text_range())
                })
                .collect();
            let prefixes = triples
                .iter()
                .map(|triple| triple.used_prefixes())
                .flatten()
                .collect();
            Some(Context {
                nodes: triples
                    .iter()
                    .map(|triple| triple.syntax())
                    .cloned()
                    .collect(),
                prefixes,
            })
        }
        _ => None,
    }
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
                {}
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
                 ?s <p2> <o2>"
            }
        );
    }
}
