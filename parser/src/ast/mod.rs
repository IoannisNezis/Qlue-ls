mod utils;

use rowan::cursor::SyntaxToken;
use utils::nth_ancestor;

use crate::{syntax_kind::SyntaxKind, SyntaxNode};

#[derive(Debug, PartialEq)]
pub struct SelectQuery {
    syntax: SyntaxNode,
}

impl SelectQuery {
    pub fn where_clause(&self) -> Option<WhereClause> {
        WhereClause::cast(self.syntax.first_child_by_kind(&WhereClause::can_cast)?)
    }
}

pub enum GroupPatternNotTriples {
    GroupOrUnionGraphPattern(GroupOrUnionGraphPattern),
    OptionalGraphPattern(OptionalGraphPattern),
    MinusGraphPattern(MinusGraphPattern),
    GraphGraphPattern(GraphGraphPattern),
    ServiceGraphPattern(ServiceGraphPattern),
    Filter(Filter),
    Bind(Bind),
    InlineData(InlineData),
}

impl GroupPatternNotTriples {
    pub fn group_graph_pattern(&self) -> Option<GraphGraphPattern> {
        match self {
            GroupPatternNotTriples::GroupOrUnionGraphPattern(_group_or_union_graph_pattern) => {
                todo!()
            }
            GroupPatternNotTriples::OptionalGraphPattern(_optional_graph_pattern) => todo!(),
            GroupPatternNotTriples::MinusGraphPattern(_minus_graph_pattern) => todo!(),
            GroupPatternNotTriples::GraphGraphPattern(_graph_graph_pattern) => todo!(),
            GroupPatternNotTriples::ServiceGraphPattern(_service_graph_pattern) => todo!(),
            GroupPatternNotTriples::Filter(_filter) => None,
            GroupPatternNotTriples::Bind(_bind) => None,
            GroupPatternNotTriples::InlineData(_inline_data) => None,
        }
    }
}

#[derive(Debug)]
pub struct GroupOrUnionGraphPattern {
    syntax: SyntaxNode,
}

#[derive(Debug)]
pub struct OptionalGraphPattern {
    syntax: SyntaxNode,
}

#[derive(Debug)]
pub struct MinusGraphPattern {
    syntax: SyntaxNode,
}

#[derive(Debug)]
pub struct GraphGraphPattern {
    syntax: SyntaxNode,
}

#[derive(Debug)]
pub struct Filter {
    syntax: SyntaxNode,
}

#[derive(Debug)]
pub struct Bind {
    syntax: SyntaxNode,
}

#[derive(Debug)]
pub struct InlineData {
    syntax: SyntaxNode,
}

#[derive(Debug)]
pub struct WhereClause {
    syntax: SyntaxNode,
}

#[derive(Debug)]
pub struct ServiceGraphPattern {
    syntax: SyntaxNode,
}

impl WhereClause {
    pub fn group_graph_pattern(&self) -> Option<GroupGraphPattern> {
        GroupGraphPattern::cast(self.syntax.first_child()?)
    }

    pub fn where_token(&self) -> Option<SyntaxToken> {
        match self.syntax.first_child_or_token() {
            Some(rowan::NodeOrToken::Token(token)) if token.kind() == SyntaxKind::WHERE => {
                Some(token.into())
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct GroupGraphPattern {
    syntax: SyntaxNode,
}

impl GroupGraphPattern {
    pub fn triple_blocks(&self) -> Vec<TriplesBlock> {
        if let Some(sub) = self
            .syntax
            .first_child_by_kind(&|kind| kind == SyntaxKind::GroupGraphPatternSub)
        {
            sub.children()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::TriplesBlock => {
                        Some(TriplesBlock::cast(child).expect("Kind should be TriplesBLock"))
                    }
                    _ => None,
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub fn r_paren_token(&self) -> Option<SyntaxToken> {
        match self.syntax.last_child_or_token() {
            Some(rowan::NodeOrToken::Token(token)) if token.kind() == SyntaxKind::RCurly => {
                Some(token.into())
            }
            _ => None,
        }
    }
    pub fn l_paren_token(&self) -> Option<SyntaxToken> {
        match self.syntax.first_child_or_token() {
            Some(rowan::NodeOrToken::Token(token)) if token.kind() == SyntaxKind::LCurly => {
                Some(token.into())
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct TriplesBlock {
    syntax: SyntaxNode,
}

impl TriplesBlock {
    /// Get the `Triple`'s contained in this `TriplesBlock`.
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

    pub fn group_graph_pattern(&self) -> Option<GroupGraphPattern> {
        GroupGraphPattern::cast(nth_ancestor(self.syntax.clone(), 2)?)
    }
}

#[derive(Debug)]
pub struct Triples {
    syntax: SyntaxNode,
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
}

#[derive(Debug)]
pub struct VarOrTerm {
    syntax: SyntaxNode,
}

impl VarOrTerm {
    pub fn is_var(&self) -> bool {
        self.syntax
            .first_child()
            .map_or(false, |child| child.kind() == SyntaxKind::Var)
    }

    pub fn is_term(&self) -> bool {
        !self.is_var()
    }
}

impl AstNode for VarOrTerm {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::VarOrTerm
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for Triples {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::TriplesSameSubjectPath
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for TriplesBlock {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::TriplesBlock
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for GroupGraphPattern {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::GroupGraphPattern
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for WhereClause {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::WhereClause
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for OptionalGraphPattern {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::OptionalGraphPattern
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for GroupOrUnionGraphPattern {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::GroupOrUnionGraphPattern
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for MinusGraphPattern {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::MinusGraphPattern
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for GraphGraphPattern {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::GraphGraphPattern
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for ServiceGraphPattern {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::ServiceGraphPattern
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for Filter {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::Filter
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for Bind {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::Bind
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for InlineData {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::InlineData
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl AstNode for SelectQuery {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::SelectQuery
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

pub trait AstNode {
    fn kind() -> SyntaxKind;

    #[inline]
    fn can_cast(kind: SyntaxKind) -> bool {
        Self::kind() == kind
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;
}

#[cfg(test)]
mod tests;
