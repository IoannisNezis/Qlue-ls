mod utils;

use rowan::cursor::SyntaxToken;
use utils::nth_ancestor;

use crate::{syntax_kind::SyntaxKind, SyntaxNode};

#[derive(Debug, PartialEq)]
pub struct QueryUnit {
    syntax: SyntaxNode,
}

impl QueryUnit {
    pub fn select_query(&self) -> Option<SelectQuery> {
        SelectQuery::cast(
            self.syntax
                .first_child()?
                .first_child_by_kind(&SelectQuery::can_cast)?,
        )
    }

    pub fn prologue(&self) -> Option<Prologue> {
        Prologue::cast(
            self.syntax
                .first_child()?
                .first_child_by_kind(&Prologue::can_cast)?,
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct Prologue {
    syntax: SyntaxNode,
}

impl Prologue {
    pub fn prefix_declarations(&self) -> Vec<PrefixDeclaration> {
        self.syntax
            .children()
            .filter_map(&PrefixDeclaration::cast)
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub struct PrefixDeclaration {
    syntax: SyntaxNode,
}

impl PrefixDeclaration {
    pub fn prefix(&self) -> Option<String> {
        Some(
            self.syntax
                .first_child_or_token_by_kind(&|kind| kind == SyntaxKind::PNAME_NS)?
                .to_string()
                .split_once(":")
                .expect("Every PNAME_NS should contain ':' at the end")
                .0
                .to_string(),
        )
    }
    pub fn uri_prefix(&self) -> Option<String> {
        Some(
            self.syntax
                .first_child_or_token_by_kind(&|kind| kind == SyntaxKind::IRIREF)?
                .to_string(),
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct SelectQuery {
    syntax: SyntaxNode,
}

impl SelectQuery {
    pub fn where_clause(&self) -> Option<WhereClause> {
        WhereClause::cast(self.syntax.first_child_by_kind(&WhereClause::can_cast)?)
    }
    pub fn select_clause(&self) -> Option<SelectClause> {
        SelectClause::cast(self.syntax.first_child_by_kind(&SelectClause::can_cast)?)
    }
    pub fn variables(&self) -> Vec<Var> {
        if let Some(where_clause) = self.where_clause() {
            if let Some(ggp) = where_clause.group_graph_pattern() {
                return ggp
                    .triple_blocks()
                    .iter()
                    .flat_map(|triple_block| {
                        triple_block
                            .triples()
                            .iter()
                            .flat_map(|triple| triple.variables())
                            .collect::<Vec<Var>>()
                    })
                    .collect();
            }
        }
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct SelectClause {
    syntax: SyntaxNode,
}

impl SelectClause {
    pub fn variables(&self) -> Vec<Var> {
        self.syntax
            .children()
            .filter_map(|child| {
                if child.kind() == SyntaxKind::Var {
                    Some(Var::cast(child).expect("Node of kind Var should be castable to Var"))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn select_query(&self) -> Option<SelectQuery> {
        SelectQuery::cast(self.syntax.parent()?)
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
    pub fn triples(&self) -> Vec<Triple> {
        self.syntax
            .children()
            .filter_map(|child| match child.kind() {
                SyntaxKind::TriplesSameSubjectPath => Some(vec![Triple::cast(child).unwrap()]),
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

#[derive(Debug, PartialEq)]
pub struct Triple {
    syntax: SyntaxNode,
}

impl Triple {
    pub fn subject(&self) -> Option<VarOrTerm> {
        self.syntax.first_child().and_then(VarOrTerm::cast)
    }

    pub fn used_prefixes(&self) -> Vec<String> {
        self.syntax
            .descendants()
            .filter_map(PrefixedName::cast)
            .map(|prefixed_name| prefixed_name.prefix())
            .collect()
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
        Some(TriplesBlock::cast(parent).expect("parent should be a TriplesBlock"))
    }

    // fn property_list_path(&self) -> Option<PropertyListPath> {
    //     let child = self.syntax.last_child()?;
    //     match child.kind() {
    //         SyntaxKind::PropertyListPathNotEmpty => PropertyListPath::cast(child),
    //         SyntaxKind::PropertyListPath => child
    //             .first_child()
    //             .map(|grand_child| PropertyListPath::cast(grand_child))
    //             .flatten(),
    //         _ => None,
    //     }
    // }

    fn variables(&self) -> Vec<Var> {
        self.syntax
            .preorder()
            .filter_map(|walk_event| match walk_event {
                rowan::WalkEvent::Enter(node) => Var::cast(node),
                rowan::WalkEvent::Leave(_) => None,
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct PropertyListPath {
    syntax: SyntaxNode,
}

impl PropertyListPath {
    pub fn variables(&self) -> Vec<Var> {
        self.syntax
            .children()
            .filter_map(|child| match child.kind() {
                SyntaxKind::VerbSimple => child.first_child().and_then(Var::cast),
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Iri {
    syntax: SyntaxNode,
}

impl Iri {
    pub fn prefixed_name(&self) -> Option<PrefixedName> {
        self.syntax.first_child().and_then(PrefixedName::cast)
    }

    pub fn is_uncompressed(&self) -> bool {
        self.syntax
            .first_child()
            .map_or(false, |child| child.kind() == SyntaxKind::IRIREF)
    }
}

#[derive(Debug)]
pub struct PrefixedName {
    syntax: SyntaxNode,
}

impl PrefixedName {
    pub fn prefix(&self) -> String {
        self.syntax
            .to_string()
            .split_once(":")
            .expect("Every PrefixedName should contain a ':'")
            .0
            .to_string()
    }
}

#[derive(Debug)]
pub struct VarOrTerm {
    syntax: SyntaxNode,
}

impl VarOrTerm {
    pub fn var(&self) -> Option<Var> {
        Var::cast(self.syntax.first_child()?)
    }

    pub fn is_var(&self) -> bool {
        self.syntax
            .first_child()
            .map_or(false, |child| child.kind() == SyntaxKind::Var)
    }

    pub fn is_term(&self) -> bool {
        !self.is_var()
    }
}

#[derive(Debug)]
pub struct Var {
    syntax: SyntaxNode,
}

impl Var {
    pub fn triple(&self) -> Option<Triple> {
        self.syntax.ancestors().find_map(Triple::cast)
    }
    pub fn var_name(&self) -> String {
        self.syntax.text().to_string()[1..].to_string()
    }
}

impl AstNode for Var {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::Var
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

impl AstNode for Iri {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::iri
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

impl AstNode for PrefixedName {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::PrefixedName
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

impl AstNode for PropertyListPath {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::PropertyListPathNotEmpty
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

impl AstNode for Triple {
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

impl AstNode for SelectClause {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::SelectClause
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

impl AstNode for Prologue {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::Prologue
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

impl AstNode for PrefixDeclaration {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::PrefixDecl
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

impl AstNode for QueryUnit {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::QueryUnit
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

    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, SyntaxKind::SelectQuery | SyntaxKind::SubSelect)
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

    fn collect_decendants(&self, matcher: &impl Fn(SyntaxKind) -> bool) -> Vec<SyntaxNode> {
        self.syntax()
            .preorder()
            .filter_map(|walk_event| match walk_event {
                rowan::WalkEvent::Enter(node) if matcher(node.kind()) => Some(node),
                _ => None,
            })
            .collect()
    }

    fn preorder_find_kind(&self, kind: SyntaxKind) -> Vec<SyntaxNode> {
        self.syntax()
            .preorder()
            .filter_map(|walk_event| match walk_event {
                rowan::WalkEvent::Enter(node) if node.kind() == kind => Some(node),
                _ => None,
            })
            .collect()
    }

    fn text(&self) -> String {
        self.syntax().text().to_string()
    }
}

#[cfg(test)]
mod tests;
