use crate::{syntax_kind::SyntaxKind, SyntaxNode};

#[derive(Debug)]
pub struct TriplesBlock {
    pub syntax: SyntaxNode,
}

impl TriplesBlock {
    pub fn triples(&self) -> Vec<Triples> {
        self.syntax
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
            SyntaxKind::TriplesBlock => Some(Self { syntax: node }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Triples {
    pub syntax: SyntaxNode,
}

impl Triples {
    pub fn subject(&self) -> Option<VarOrTerm> {
        self.syntax
            .first_child()
            .map(|child| VarOrTerm::cast(child))
            .flatten()
    }

    /// Get the `TriplesBlock` this Triple is part of.
    /// **Note** that this referes to the topmost TriplesBlock and not the next.
    pub fn triples_block(&self) -> Option<TriplesBlock> {
        let mut parent = self.syntax.parent()?;
        if parent.kind() != SyntaxKind::TriplesBlock {
            return None;
        }
        while let Some(node) = parent.parent() {
            if node.kind() == SyntaxKind::TriplesBlock {
                parent = node;
            } else {
                break;
            }
        }
        return Some(TriplesBlock::cast(parent).expect("parent should be a TriplesBlock"));
    }

    pub fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::TriplesSameSubjectPath => Some(Self { syntax: node }),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct VarOrTerm {
    pub syntax: SyntaxNode,
}

impl VarOrTerm {
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
            SyntaxKind::VarOrTerm => Some(Self { syntax: node }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
