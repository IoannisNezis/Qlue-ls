use std::collections::HashSet;

use ll_sparql_parser::{
    ast::{AstNode, GraphPatternNotTriples, GroupGraphPattern, PrefixedName, Triple},
    SyntaxNode,
};
use serde::Serialize;
use text_size::TextSize;

use super::{query_graph::QueryGraph, CompletionLocation};

#[derive(Debug, Clone, Default)]
pub(crate) struct Context {
    pub nodes: Vec<SyntaxNode>,
    pub prefixes: HashSet<String>,
    pub raw_inject: String,
}

impl Serialize for Context {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match (!self.nodes.is_empty(), !self.raw_inject.is_empty()) {
            (false, false) => serializer.serialize_none(),
            (false, true) => serializer.serialize_str(&format!("{{{}}}", self.raw_inject)),
            (true, false) => serializer.serialize_str(&format!(
                "{{{}}}",
                self.nodes
                    .iter()
                    .map(|node| node.to_string())
                    .collect::<Vec<_>>()
                    .join(" .\n")
            )),

            (true, true) => serializer.serialize_str(&format!(
                "{{{}{}}}",
                self.nodes
                    .iter()
                    .map(|node| node.to_string())
                    .collect::<Vec<_>>()
                    .join(" .\n"),
                if self.raw_inject.is_empty() {
                    String::new()
                } else {
                    format!(". {}", &self.raw_inject)
                }
            )),
        }
    }
}

pub(super) fn context(location: &CompletionLocation) -> Option<Context> {
    match location {
        CompletionLocation::Predicate(triple) | CompletionLocation::Object(triple) => {
            compute_context(triple)
        }
        _ => None,
    }
}

fn compute_context(triple: &Triple) -> Option<Context> {
    let mut graph = QueryGraph::new();
    // NOTE: this ensures that the trigger triple is node no. 0
    graph.add_node(
        triple.syntax().clone(),
        triple.variables().iter().map(|var| var.text()).collect(),
    );

    collect_nodes(
        &mut graph,
        triple.triples_block()?.group_graph_pattern()?,
        triple.syntax().text_range().start(),
    );
    // NOTE: compute edges based on visible variables of each pattern
    graph.connect();

    let mut nodes: Vec<SyntaxNode> = graph
        .component(0)
        .into_iter()
        .filter(|node| node.text_range() != triple.syntax().text_range())
        .collect();
    nodes.sort_by_key(|node| node.text_range().start());
    let prefixes: HashSet<String> = nodes
        .iter()
        .flat_map(|node| {
            node.descendants()
                .filter_map(PrefixedName::cast)
                .map(|prefixed_name| prefixed_name.prefix())
        })
        .collect();
    Some(Context {
        nodes,
        prefixes,
        raw_inject: String::new(),
    })
}

fn collect_nodes(graph: &mut QueryGraph, group_graph_pattern: GroupGraphPattern, cutoff: TextSize) {
    // NOTE: add all triples in the **current** group_graph_pattern (including sup patterns)
    for triple in group_graph_pattern
        .triple_blocks()
        .into_iter()
        .flat_map(|triples_block| triples_block.triples())
        .filter(|other_triple| {
            other_triple.syntax().text_range().start() < cutoff && !other_triple.has_error()
        })
    {
        graph.add_node(
            triple.syntax().clone(),
            triple.variables().iter().map(|var| var.text()).collect(),
        );
    }

    // NOTE: add non triples patterns (filters, sub-pattern and so on)
    for pattern in group_graph_pattern
        .group_pattern_not_triples()
        .into_iter()
        .filter(|pattern| pattern.syntax().text_range().start() < cutoff && !pattern.has_error())
    {
        match pattern {
            GraphPatternNotTriples::GroupOrUnionGraphPattern(pattern)
                if pattern.syntax().children().count() == 1 =>
            {
                collect_nodes(
                    graph,
                    GroupGraphPattern::cast(pattern.syntax().first_child().unwrap()).unwrap(),
                    cutoff,
                );
            }
            _ => graph.add_node(
                pattern.syntax().clone(),
                pattern
                    .visible_variables()
                    .iter()
                    .map(|var| var.text())
                    .collect(),
            ),
        };
    }
    // NOTE: This GroupGraphPattern could be a sub select
    if let Some(sub_select) = group_graph_pattern.sub_select() {
        graph.add_node(
            group_graph_pattern.syntax().clone(),
            sub_select
                .visible_variables()
                .iter()
                .map(|var| var.text())
                .collect(),
        );
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
        let offset = cursor.byte_index(input).unwrap();
        let trigger_token = get_trigger_token(&root, offset).unwrap();
        let anchor = get_anchor_token(trigger_token, offset);
        let continuations = get_continuations(&root, &anchor);
        let location = get_location(&anchor, &continuations, offset);
        context(&location).unwrap()
    }

    #[test]
    fn context_simple() {
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
            serde_json::to_value(&context).unwrap().as_str().unwrap(),
            indoc! {
                "{?s <p1> <o1> .
                 ?s <p2> <o2>}"
            }
        );
    }

    #[test]
    fn context_unconnected() {
        let input = indoc! {
            "Select * {
                ?x <p1> <o1> .
                ?s <p2> <o2> .
                ?s 
             }
            "
        };
        let position = Position::new(3, 6);
        let context = location_at(input, position);
        assert_eq!(
            serde_json::to_value(&context).unwrap().as_str().unwrap(),
            indoc! {
                "{?s <p2> <o2>}"
            }
        );
    }

    #[test]
    fn context_filter() {
        let input = indoc! {
            "Select * {
               ?n1 <p1> ?n2  FILTER (?n2)
               ?n2 <p2> <o2> FILTER (?n3)
               ?n1 
             }
            "
        };
        let position = Position::new(3, 6);
        let context = location_at(input, position);
        assert_eq!(
            serde_json::to_value(&context).unwrap().as_str().unwrap(),
            indoc! {
                "{?n1 <p1> ?n2 .
                 FILTER (?n2) .
                 ?n2 <p2> <o2>}"
            }
        );
    }

    #[test]
    fn context_sub_pattern() {
        let input = indoc! {
            "Select * {
               {
                 {
                    ?n1 <> ?n2 .
                    ?n3 <> <>
                 }
                 ?n2 <> <>
               }
               ?n1 
             }
            "
        };
        let position = Position::new(8, 6);
        let context = location_at(input, position);
        assert_eq!(
            serde_json::to_value(&context).unwrap().as_str().unwrap(),
            indoc! {
                "{?n1 <> ?n2 .
                 ?n2 <> <>}"
            }
        );
    }

    #[test]
    fn context_sub_select() {
        let input = indoc! {
            "Select * {
             {
              Select * WHERE {
                ?n1 <> <>
              }
             }
             {
              Select * WHERE {
                ?n2 <> <>
              }
             }
             {
              Select ?n1 WHERE {
                ?n2 <> <>
              }
             }
             {
              Select ?n2 WHERE {
                ?n1 <> <>
              }
             }
             {
              Select (?n1 as ?n2) WHERE {
                ?n1 <> <>
              }
             }
             {
              Select (?n2 as ?n1) WHERE {
                ?n1 <> <>
              }
             }
             ?n1 
             }"
        };
        let position = Position::new(31, 4);
        let context = location_at(input, position);
        assert_eq!(
            serde_json::to_value(&context).unwrap().as_str().unwrap(),
            indoc! {
                "{{
                  Select * WHERE {
                    ?n1 <> <>
                  }
                 } .
                 {
                  Select ?n1 WHERE {
                    ?n2 <> <>
                  }
                 } .
                 {
                  Select (?n2 as ?n1) WHERE {
                    ?n1 <> <>
                  }
                 }}"
            }
        );
    }

    #[test]
    fn context_complex() {
        let input = indoc! {
            r#"Select * {
                 ?n1 <p1> ?n2 .
                 ?n6 <> ?n4 .
                 ?n4 <p2> ?n3 .
                 ?n4 <> ?n9 .
                 ?n5 ?n6 "dings" .
                 ?n7 <> ?n8 .
                 ?n8 <> ?n7 .
                 ?n4 <> ?n2 .
                 ?n1 
               }
            "#
        };
        let position = Position::new(9, 6);
        let context = location_at(input, position);
        assert_eq!(
            serde_json::to_value(&context).unwrap().as_str().unwrap(),
            indoc! {
              r#"{?n1 <p1> ?n2 .
                 ?n6 <> ?n4 .
                 ?n4 <p2> ?n3 .
                 ?n4 <> ?n9 .
                 ?n5 ?n6 "dings" .
                 ?n4 <> ?n2}"#
            }
        );
    }

    // #[test]
    // fn context_super_block() {
    //     let input = indoc! {
    //         "Select * {
    //             ?s <p1> <o1>
    //             {
    //               ?s
    //             }
    //          }
    //         "
    //     };
    //     let position = Position::new(3, 8);
    //     let context = location_at(input, position);
    //     assert_eq!(
    //         serde_json::to_value(&context).unwrap().as_str().unwrap(),
    //         indoc! {
    //             "?s <p1> <o1>"
    //         }
    //     );
    // }
}
