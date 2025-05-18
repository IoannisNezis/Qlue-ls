use std::collections::{HashMap, HashSet};

use ll_sparql_parser::SyntaxNode;

#[derive(Debug)]
pub(super) struct QueryGraph {
    nodes: Vec<QueryGraphNode>,
    adjacency_list: HashMap<u32, HashSet<u32>>,
}

impl QueryGraph {
    pub(super) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            adjacency_list: HashMap::new(),
        }
    }

    pub(super) fn add_node(&mut self, syntax: SyntaxNode, visible_variables: HashSet<String>) {
        self.adjacency_list
            .insert(self.nodes.len() as u32, HashSet::new());
        self.nodes.push(QueryGraphNode {
            syntax,
            visible_variables,
        });
    }

    pub(super) fn add_edge(&mut self, u: u32, v: u32) {
        self.adjacency_list.entry(u).or_default().insert(v);
        self.adjacency_list.entry(v).or_default().insert(u);
    }

    pub(super) fn connect(&mut self) {
        for u in 0..self.nodes.len() {
            for v in (u + 1)..self.nodes.len() {
                if self.nodes[u]
                    .visible_variables
                    .iter()
                    .any(|var| self.nodes[v].visible_variables.contains(var))
                {
                    self.add_edge(u as u32, v as u32);
                }
            }
        }
    }

    pub(super) fn component(&self, u: u32) -> Vec<SyntaxNode> {
        let mut visited: HashSet<u32> = HashSet::from_iter([u]);
        let mut stack: Vec<u32> = Vec::from_iter(
            self.adjacency_list
                .get(&u)
                .expect("start node should be in nodes")
                .iter()
                .cloned(),
        );
        while let Some(node) = stack.pop() {
            stack.extend(self.adjacency_list.get(&node).unwrap() - &visited);
            visited.insert(node);
        }
        visited
            .into_iter()
            .map(|node| self.nodes[node as usize].syntax.clone())
            .collect()
    }
}

#[derive(Debug)]
struct QueryGraphNode {
    syntax: SyntaxNode,
    visible_variables: HashSet<String>,
}
