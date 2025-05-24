mod utils;

use std::usize;

use rowan::{cursor::SyntaxToken, SyntaxNodeChildren, TextSize};
use utils::nth_ancestor;

use crate::{syntax_kind::SyntaxKind, Sparql, SyntaxNode};

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
pub struct SolutionModifier {
    syntax: SyntaxNode,
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
        vec![]
    }

    pub fn soulution_modifier(&self) -> Option<SolutionModifier> {
        SolutionModifier::cast(self.syntax.last_child()?)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SelectClause {
    syntax: SyntaxNode,
}

impl SelectClause {
    pub fn variables(&self) -> Vec<Var> {
        self.syntax.children().filter_map(Var::cast).collect()
    }

    pub fn select_query(&self) -> Option<SelectQuery> {
        SelectQuery::cast(self.syntax.parent()?)
    }
}

#[derive(Debug)]
pub enum GraphPatternNotTriples {
    GroupOrUnionGraphPattern(GroupOrUnionGraphPattern),
    OptionalGraphPattern(OptionalGraphPattern),
    MinusGraphPattern(MinusGraphPattern),
    GraphGraphPattern(GraphGraphPattern),
    ServiceGraphPattern(ServiceGraphPattern),
    Filter(Filter),
    Bind(Bind),
    InlineData(InlineData),
}

impl GraphPatternNotTriples {
    pub fn group_graph_pattern(&self) -> Option<GraphGraphPattern> {
        match self {
            GraphPatternNotTriples::GroupOrUnionGraphPattern(_group_or_union_graph_pattern) => {
                todo!()
            }
            GraphPatternNotTriples::OptionalGraphPattern(_optional_graph_pattern) => todo!(),
            GraphPatternNotTriples::MinusGraphPattern(_minus_graph_pattern) => todo!(),
            GraphPatternNotTriples::GraphGraphPattern(_graph_graph_pattern) => todo!(),
            GraphPatternNotTriples::ServiceGraphPattern(_service_graph_pattern) => todo!(),
            GraphPatternNotTriples::Filter(_filter) => None,
            GraphPatternNotTriples::Bind(_bind) => None,
            GraphPatternNotTriples::InlineData(_inline_data) => None,
        }
    }
}

#[derive(Debug)]
pub struct GroupOrUnionGraphPattern {
    syntax: SyntaxNode,
}
impl GroupOrUnionGraphPattern {
    fn group_graph_patterns(&self) -> Vec<GroupGraphPattern> {
        self.syntax
            .children()
            .filter_map(GroupGraphPattern::cast)
            .collect()
    }
}

#[derive(Debug)]
pub struct OptionalGraphPattern {
    syntax: SyntaxNode,
}
impl OptionalGraphPattern {
    fn group_graph_pattern(&self) -> Option<GroupGraphPattern> {
        self.syntax.last_child().and_then(GroupGraphPattern::cast)
    }
}

#[derive(Debug)]
pub struct MinusGraphPattern {
    syntax: SyntaxNode,
}
impl MinusGraphPattern {
    fn group_graph_pattern(&self) -> Option<GroupGraphPattern> {
        self.syntax.last_child().and_then(GroupGraphPattern::cast)
    }
}

#[derive(Debug)]
pub struct GraphGraphPattern {
    syntax: SyntaxNode,
}
impl GraphGraphPattern {
    fn group_graph_pattern(&self) -> Option<GroupGraphPattern> {
        self.syntax.last_child().and_then(GroupGraphPattern::cast)
    }
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

impl ServiceGraphPattern {
    pub fn iri(&self) -> Option<Iri> {
        self.syntax
            .children()
            .find(|child| child.kind() == SyntaxKind::VarOrIri)
            .and_then(|child| child.first_child().and_then(Iri::cast))
    }

    pub fn group_graph_pattern(&self) -> Option<GroupGraphPattern> {
        self.syntax
            .children()
            .last()
            .and_then(GroupGraphPattern::cast)
    }
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
        self.syntax()
            .first_child_by_kind(&|kind| kind == SyntaxKind::GroupGraphPatternSub)
            .map(|ggp| ggp.children().filter_map(TriplesBlock::cast).collect())
            .unwrap_or_default()
    }

    pub fn group_pattern_not_triples(&self) -> Vec<GraphPatternNotTriples> {
        self.syntax()
            .first_child_by_kind(&|kind| kind == SyntaxKind::GroupGraphPatternSub)
            .map(|ggp| {
                ggp.children()
                    .filter_map(GraphPatternNotTriples::cast)
                    .collect()
            })
            .unwrap_or_default()
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

    pub fn sub_select(&self) -> Option<SelectQuery> {
        self.syntax().first_child().and_then(SelectQuery::cast)
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
pub struct Subject {
    syntax: SyntaxNode,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Triple {
    syntax: SyntaxNode,
}

impl Triple {
    pub fn subject(&self) -> Option<Subject> {
        self.syntax.first_child().and_then(Subject::cast)
    }

    pub fn properties_list_path(&self) -> Option<PropertyListPath> {
        PropertyListPath::cast(self.syntax.children().nth(1)?)
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

    pub fn variables(&self) -> Vec<Var> {
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
pub struct PropertyPath {
    pub verb: Path,
    pub object: ObjectList,
}

impl PropertyPath {
    pub fn text(&self) -> String {
        format!("{} {}", self.verb.text(), self.object.text())
    }
}

#[derive(Debug)]
pub struct Path {
    syntax: SyntaxNode,
}

impl Path {
    pub fn sub_paths(&self) -> SubPaths {
        SubPaths {
            children: self.syntax.children(),
        }
    }
}

pub struct SubPaths {
    children: SyntaxNodeChildren<Sparql>,
}

impl Iterator for SubPaths {
    type Item = Path;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next_child) = self.children.next() {
            if let Some(path) = Path::cast(next_child.into()) {
                return Some(path);
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct ObjectList {
    syntax: SyntaxNode,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlankPropertyList {
    syntax: SyntaxNode,
}

impl BlankPropertyList {
    pub fn triple(&self) -> Option<Triple> {
        match self.syntax.kind() {
            SyntaxKind::BlankNodePropertyListPath => {
                todo!()
            }
            SyntaxKind::BlankNodePropertyList => {
                todo!()
            }
            SyntaxKind::BlankNode => self.syntax.ancestors().nth(7).and_then(Triple::cast),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        todo!()
    }

    pub fn is_subject(&self) -> bool {
        todo!()
    }

    pub fn property_list(&self) -> Option<PropertyListPath> {
        match self.syntax.kind() {
            SyntaxKind::BlankNodePropertyListPath | SyntaxKind::BlankNodePropertyList => {
                PropertyListPath::cast(self.syntax.first_child()?)
            }

            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct PropertyListPath {
    syntax: SyntaxNode,
}

impl PropertyListPath {
    pub fn properties(&self) -> Vec<PropertyPath> {
        self.syntax
            .children()
            .step_by(2)
            .filter_map(|child| {
                match (
                    Path::cast(child.clone()),
                    child.next_sibling().and_then(ObjectList::cast),
                ) {
                    (None, None) | (None, Some(_)) | (Some(_), None) => None,
                    (Some(path), Some(object_list)) => Some(PropertyPath {
                        verb: path,
                        object: object_list,
                    }),
                }
            })
            .collect()
    }
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

    /// Converts a IRIREF "<abc>" into the raw string "abc"
    /// Returns None this iri is a PrefixedName
    pub fn raw_iri(&self) -> Option<String> {
        (self.syntax.first_child_or_token()?.kind() == SyntaxKind::IRIREF
            && self.syntax.text_range().len() >= 2.into())
        .then(|| self.text()[1..usize::from(self.syntax.text_range().len()) - 1].to_string())
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

    pub fn name(&self) -> String {
        self.syntax
            .to_string()
            .split_once(":")
            .expect("Every PrefixedName should contain a ':'")
            .1
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

    fn visible_variables(&self) -> Vec<Var> {
        vec![Var::cast(self.syntax().clone()).unwrap()]
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

    fn visible_variables(&self) -> Vec<Var> {
        self.syntax
            .first_child()
            .and_then(Var::cast)
            .map(|var| vec![var])
            .unwrap_or_default()
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

    fn visible_variables(&self) -> Vec<Var> {
        vec![]
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

    fn visible_variables(&self) -> Vec<Var> {
        vec![]
    }
}

impl AstNode for Path {
    fn kind() -> SyntaxKind {
        SyntaxKind::VerbPath
    }

    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::Path
                | SyntaxKind::VerbPath
                | SyntaxKind::PathAlternative
                | SyntaxKind::PathSequence
                | SyntaxKind::PathElt
                | SyntaxKind::PathEltOrInverse
                | SyntaxKind::PathPrimary
                | SyntaxKind::PathNegatedPropertySet
                | SyntaxKind::PathOneInPropertySet
        )
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }

    fn visible_variables(&self) -> Vec<Var> {
        vec![]
    }
}

impl AstNode for ObjectList {
    fn kind() -> SyntaxKind {
        SyntaxKind::ObjectListPath
    }
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, SyntaxKind::ObjectListPath | SyntaxKind::ObjectList)
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }

    fn visible_variables(&self) -> Vec<Var> {
        self.syntax.descendants().filter_map(Var::cast).collect()
    }
}

impl AstNode for BlankPropertyList {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::BlankNodePropertyListPath
    }

    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::BlankNodePropertyListPath
                | SyntaxKind::BlankNodePropertyList
                | SyntaxKind::BlankNode
        )
    }

    #[inline]
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

    fn visible_variables(&self) -> Vec<Var> {
        self.syntax.descendants().filter_map(Var::cast).collect()
    }
}

impl AstNode for PropertyListPath {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::PropertyListPathNotEmpty
    }

    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::PropertyListPath | SyntaxKind::PropertyListPathNotEmpty
        )
    }

    #[inline]
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

    fn visible_variables(&self) -> Vec<Var> {
        self.syntax.descendants().filter_map(Var::cast).collect()
    }
}

impl AstNode for Subject {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::VarOrTerm
    }

    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, SyntaxKind::VarOrTerm | SyntaxKind::TriplesNodePath)
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

    fn visible_variables(&self) -> Vec<Var> {
        self.syntax
            .first_child()
            .and_then(Var::cast)
            .map(|var| vec![var])
            .unwrap_or_default()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.syntax.descendants().filter_map(Var::cast).collect()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.syntax.descendants().filter_map(Var::cast).collect()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.sub_select()
            .map(|select| select.visible_variables())
            .or(self.syntax.first_child().map(|child| {
                child
                    .children()
                    .into_iter()
                    .filter_map(|child| match child.kind() {
                        SyntaxKind::TriplesBlock => {
                            TriplesBlock::cast(child).map(|tp| tp.visible_variables())
                        }
                        SyntaxKind::GraphPatternNotTriples => GraphPatternNotTriples::cast(child)
                            .map(|pattern| pattern.visible_variables()),
                        _ => None,
                    })
                    .flatten()
                    .collect()
            }))
            .unwrap_or_default()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.group_graph_pattern()
            .map(|ggp| ggp.visible_variables())
            .unwrap_or_default()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.group_graph_pattern()
            .map(|ggp| ggp.visible_variables())
            .unwrap_or_default()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.group_graph_patterns()
            .into_iter()
            .flat_map(|ggp| ggp.visible_variables())
            .collect()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.group_graph_pattern()
            .map(|ggp| ggp.visible_variables())
            .unwrap_or_default()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.group_graph_pattern()
            .map(|ggp| ggp.visible_variables())
            .unwrap_or_default()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.group_graph_pattern()
            .map(|ggp| ggp.visible_variables())
            .unwrap_or_default()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.syntax().descendants().filter_map(Var::cast).collect()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.syntax()
            .children()
            .last()
            .and_then(Var::cast)
            .map(|var| vec![var])
            .unwrap_or_default()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.syntax().descendants().filter_map(Var::cast).collect()
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

    fn visible_variables(&self) -> Vec<Var> {
        self.variables()
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

    fn visible_variables(&self) -> Vec<Var> {
        vec![]
    }
}

impl AstNode for SolutionModifier {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::SolutionModifier
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

    fn visible_variables(&self) -> Vec<Var> {
        vec![]
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

    fn visible_variables(&self) -> Vec<Var> {
        vec![]
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

    fn visible_variables(&self) -> Vec<Var> {
        self.select_query()
            .map(|select_query| select_query.visible_variables())
            .unwrap_or_default()
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

    fn visible_variables(&self) -> Vec<Var> {
        if let Some(select_clause) = self.select_clause() {
            if let Some(SyntaxKind::Star) = select_clause
                .syntax
                .last_child_or_token()
                .map(|last| last.kind())
            {
                self.where_clause()
                    .map(|where_clause| where_clause.visible_variables())
                    .unwrap_or_default()
            } else {
                select_clause.variables()
            }
        } else {
            Vec::new()
        }
    }
}

impl AstNode for GraphPatternNotTriples {
    #[inline]
    fn kind() -> SyntaxKind {
        SyntaxKind::GraphPatternNotTriples
    }

    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::GroupOrUnionGraphPattern
                | SyntaxKind::OptionalGraphPattern
                | SyntaxKind::MinusGraphPattern
                | SyntaxKind::GraphGraphPattern
                | SyntaxKind::ServiceGraphPattern
                | SyntaxKind::Filter
                | SyntaxKind::Bind
                | SyntaxKind::InlineData
        )
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let child = syntax.first_child()?;
        match child.kind() {
            SyntaxKind::GroupOrUnionGraphPattern => {
                Some(GraphPatternNotTriples::GroupOrUnionGraphPattern(
                    GroupOrUnionGraphPattern::cast(child)?,
                ))
            }
            SyntaxKind::OptionalGraphPattern => Some(GraphPatternNotTriples::OptionalGraphPattern(
                OptionalGraphPattern::cast(child)?,
            )),
            SyntaxKind::MinusGraphPattern => Some(GraphPatternNotTriples::MinusGraphPattern(
                MinusGraphPattern::cast(child)?,
            )),
            SyntaxKind::GraphGraphPattern => Some(GraphPatternNotTriples::GraphGraphPattern(
                GraphGraphPattern::cast(child)?,
            )),
            SyntaxKind::ServiceGraphPattern => Some(GraphPatternNotTriples::ServiceGraphPattern(
                ServiceGraphPattern::cast(child)?,
            )),
            SyntaxKind::Filter => Some(GraphPatternNotTriples::Filter(Filter::cast(child)?)),
            SyntaxKind::Bind => Some(GraphPatternNotTriples::Bind(Bind::cast(child)?)),
            SyntaxKind::InlineData => Some(GraphPatternNotTriples::InlineData(InlineData::cast(
                syntax,
            )?)),
            _ => None,
        }
    }

    #[inline]
    fn syntax(&self) -> &SyntaxNode {
        match self {
            GraphPatternNotTriples::GroupOrUnionGraphPattern(x) => x.syntax(),
            GraphPatternNotTriples::OptionalGraphPattern(x) => x.syntax(),
            GraphPatternNotTriples::MinusGraphPattern(x) => x.syntax(),
            GraphPatternNotTriples::GraphGraphPattern(x) => x.syntax(),
            GraphPatternNotTriples::ServiceGraphPattern(x) => x.syntax(),
            GraphPatternNotTriples::Filter(x) => x.syntax(),
            GraphPatternNotTriples::Bind(x) => x.syntax(),
            GraphPatternNotTriples::InlineData(x) => x.syntax(),
        }
    }

    fn visible_variables(&self) -> Vec<Var> {
        match self {
            GraphPatternNotTriples::GroupOrUnionGraphPattern(x) => x.visible_variables(),
            GraphPatternNotTriples::OptionalGraphPattern(x) => x.visible_variables(),
            GraphPatternNotTriples::MinusGraphPattern(x) => x.visible_variables(),
            GraphPatternNotTriples::GraphGraphPattern(x) => x.visible_variables(),
            GraphPatternNotTriples::ServiceGraphPattern(x) => x.visible_variables(),
            GraphPatternNotTriples::Filter(x) => x.visible_variables(),
            GraphPatternNotTriples::Bind(x) => x.visible_variables(),
            GraphPatternNotTriples::InlineData(x) => x.visible_variables(),
        }
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

    fn visible_variables(&self) -> Vec<Var>;

    fn has_error(&self) -> bool {
        self.syntax()
            .preorder()
            .find(|walk_event| match walk_event {
                rowan::WalkEvent::Enter(node) if node.kind() == SyntaxKind::Error => true,
                _ => false,
            })
            .is_some()
    }

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

    fn used_prefixes(&self) -> Vec<String> {
        self.syntax()
            .descendants()
            .filter_map(PrefixedName::cast)
            .map(|prefixed_name| prefixed_name.prefix())
            .collect()
    }

    fn text(&self) -> String {
        self.syntax().text().to_string()
    }

    fn text_until(&self, offset: TextSize) -> String {
        let syntax = self.syntax();
        assert!(syntax.text_range().start() <= offset);
        syntax
            .text()
            .slice(TextSize::new(0)..(offset - syntax.text_range().start()))
            .to_string()
    }
}

#[cfg(test)]
mod tests;
