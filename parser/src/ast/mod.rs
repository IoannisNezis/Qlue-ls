use crate::{print_full_tree, syntax_kind::SyntaxKind, SyntaxNode};

#[derive(Debug)]
pub struct TriplesBlock {
    raw: SyntaxNode,
}

impl TriplesBlock {
    pub fn triples(&self) -> Vec<Triples> {
        println!("{}", print_full_tree(&self.raw, 0));
        self.raw
            .children()
            .filter_map(|child| match child.kind() {
                SyntaxKind::TriplesSameSubjectPath => Some(vec![Triples::cast(child).unwrap()]),
                SyntaxKind::TriplesBlock => Some(TriplesBlock::cast(child).unwrap().triples()),
                _ => None,
            })
            .flatten()
            .collect()
    }
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::TriplesBlock => Some(Self { raw: node }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Triples {
    raw: SyntaxNode,
}

impl Triples {
    pub fn subject(&self) -> Option<VarOrTerm> {
        self.raw
            .first_child()
            .map(|child| VarOrTerm::cast(child))
            .flatten()
    }

    pub fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::TriplesSameSubjectPath => Some(Self { raw: node }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct VarOrTerm {
    raw: SyntaxNode,
}

impl VarOrTerm {
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::VarOrTerm => Some(Self { raw: node }),
            _ => None,
        }
    }
}

mod test;
