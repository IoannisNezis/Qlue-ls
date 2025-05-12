use core::fmt;
use lazy_static::lazy_static;
use ll_sparql_parser::ast::QueryUnit;
use std::result;
use std::{collections::HashSet, vec};

use ll_sparql_parser::{
    parse_query, print_full_tree, syntax_kind::SyntaxKind, SyntaxElement, SyntaxElementChildren,
    SyntaxNode,
};
use text_size::{TextRange, TextSize};

use crate::server::{
    configuration::FormatSettings,
    lsp::{
        errors::LSPError,
        textdocument::{Position, Range, TextDocumentItem, TextEdit},
        FormattingOptions,
    },
};

use super::utils::KEYWORDS;

pub(super) fn format_document(
    document: &TextDocumentItem,
    options: &FormattingOptions,
    settings: &FormatSettings,
) -> Result<Vec<TextEdit>, LSPError> {
    let mut settings = settings.clone();
    settings.insert_spaces.unwrap_or(options.insert_spaces);
    let indent_string = match settings.insert_spaces.unwrap_or(options.insert_spaces) {
        true => " ".repeat(settings.tab_size.unwrap_or(options.tab_size) as usize),
        false => "\t".to_string(),
    };
    let walker = Walker::new(
        parse_query(&document.text),
        &document.text,
        settings,
        indent_string,
    );

    let mut siplified_edits: Vec<_> = walker.into_iter().flatten().collect();

    siplified_edits.sort_by(|a, b| {
        b.range
            .end()
            .cmp(&a.range.end())
            .then_with(|| b.range.start().cmp(&a.range.start()))
    });
    let mut edits = transform_edits(siplified_edits, &document.text);
    edits.sort_by(|a, b| {
        b.range
            .start
            .cmp(&a.range.start)
            .then_with(|| b.range.end.cmp(&a.range.end))
    });
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));
    edits = consolidate_edits(edits)
        .into_iter()
        .map(|ce| ce.fuse())
        .collect();
    edits = remove_redundent_edits(edits, document);

    Ok(edits)
}

#[derive(Debug)]
struct SimplifiedTextEdit {
    range: TextRange,
    text: String,
}

impl SimplifiedTextEdit {
    fn new(range: TextRange, text: &str) -> Self {
        Self {
            range,
            text: text.to_string(),
        }
    }
}

#[derive(Debug)]
struct CommentMarker {
    text: String,
    position: Position,
    indentation_level: usize,
    trailing: bool,
}

lazy_static! {
    static ref INC_INDENTATION: HashSet<SyntaxKind> = HashSet::from([
        SyntaxKind::BlankNodePropertyListPath,
        SyntaxKind::GroupGraphPattern,
        // SyntaxKind::TriplesTemplateBlock,
        SyntaxKind::BrackettedExpression,
        SyntaxKind::ConstructTemplate,
        SyntaxKind::QuadData,
    ]);
}

struct Walker<'a> {
    text: &'a str,
    queue: Vec<(SyntaxElement, u8)>,
    indent_base: String, // state:
    settings: &'a FormatSettings,
}
impl<'a> Walker<'a> {
    fn new(
        root: SyntaxNode,
        text: &'a str,
        settings: &'a FormatSettings,
        indent_base: String,
    ) -> Self {
        Self {
            queue: vec![(SyntaxElement::Node(root), 0)],
            indent_base,
            text,
            settings,
        }
    }

    fn node_augmentation(
        &self,
        node: &SyntaxElement,
        children: &Vec<SyntaxElement>,
        indentation: u8,
    ) -> Vec<SimplifiedTextEdit> {
        let mut augmentations = self.in_node_augmentation(node, children, indentation);

        if let Some(edits) = self.pre_node_augmentation(node, indentation) {
            augmentations.push(edits);
        }
        if let Some(edits) = self.post_node_augmentation(node, indentation) {
            augmentations.push(edits);
        }

        // NOTE: Capitalize keywords
        if KEYWORDS.contains(&node.kind()) && self.settings.capitalize_keywords {
            augmentations.push(SimplifiedTextEdit::new(
                node.text_range(),
                &node.to_string().to_uppercase(),
            ));
        }
        augmentations
    }

    fn in_node_augmentation(
        &self,
        node: &SyntaxElement,
        children: &Vec<SyntaxElement>,
        indentation: u8,
    ) -> Vec<SimplifiedTextEdit> {
        match node.kind() {
            SyntaxKind::QueryUnit => match (children.first(), children.last()) {
                (Some(first), Some(last)) => vec![
                    SimplifiedTextEdit::new(
                        TextRange::new(0.into(), first.text_range().start()),
                        "",
                    ),
                    SimplifiedTextEdit::new(
                        TextRange::new(last.text_range().end(), node.text_range().end()),
                        "\n",
                    ),
                ],
                _ => vec![SimplifiedTextEdit::new(
                    TextRange::new(0.into(), node.text_range().end()),
                    "",
                )],
            },
            SyntaxKind::Prologue if self.settings.align_prefixes => {
                let prefix_pos_and_length: Vec<(TextSize, TextSize)> = children
                    .iter()
                    .filter_map(|child| {
                        match (
                            child.kind(),
                            child.as_node().and_then(|child_node| {
                                child_node
                                    .children_with_tokens()
                                    .filter(|child_child| !child_child.kind().is_trivia())
                                    .nth(1)
                            }),
                        ) {
                            (SyntaxKind::PrefixDecl, Some(grandchild))
                                if grandchild.kind() == SyntaxKind::PNAME_NS =>
                            {
                                Some((grandchild.text_range().end(), grandchild.text_range().len()))
                            }
                            _ => None,
                        }
                    })
                    .collect();
                let max_length = prefix_pos_and_length
                    .iter()
                    .map(|(_pos, len)| *len)
                    .max()
                    .unwrap_or(0.into());
                prefix_pos_and_length
                    .into_iter()
                    .map(|(position, length)| {
                        SimplifiedTextEdit::new(
                            TextRange::empty(position),
                            &" ".repeat((max_length - length).into()),
                        )
                    })
                    .collect()
            }

            SyntaxKind::SelectClause | SyntaxKind::GroupCondition => children
                .iter()
                .enumerate()
                .filter_map(|(idx, child)| match child.kind() {
                    SyntaxKind::RParen | SyntaxKind::SELECT => None,
                    SyntaxKind::LParen if idx == 0 => None,
                    _ if child
                        .prev_sibling_or_token()
                        .map_or(false, |prev| prev.kind() == SyntaxKind::LParen) =>
                    {
                        None
                    }
                    _ if idx > 0
                        && children
                            .get(idx - 1)
                            .map_or(false, |prev| prev.kind() == SyntaxKind::LParen) =>
                    {
                        None
                    }
                    _ if idx > 0 => Some(SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().start()),
                        " ",
                    )),
                    _ => None,
                })
                .collect(),
            SyntaxKind::ConstructQuery => children
                .iter()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::CONSTRUCT => Some(SimplifiedTextEdit::new(
                        TextRange::new(child.text_range().end(), child.text_range().end()),
                        " ",
                    )),
                    SyntaxKind::LCurly => Some(SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().start()),
                        " ",
                    )),
                    SyntaxKind::RCurly => Some(SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().start()),
                        "\n",
                    )),
                    _ => None,
                })
                .collect(),

            SyntaxKind::PropertyListPathNotEmpty => match node.parent().map(|parent| parent.kind())
            {
                // Some(SyntaxKind::BlankNodePropertyListPath) => children
                //     .iter()
                //     .enumerate()
                //     .filter_map(|(idx, child)| match child.kind() {
                //         SyntaxKind::Semicolon if idx < children.len() - 1 => {
                //             let linebreak = self.get_linebreak(indentation);
                //             Some(SimplifiedTextEdit::new(
                //                 TextRange::empty(child.text_range().end()),
                //                 &linebreak[..linebreak.len() - 1],
                //             ))
                //         }
                //         _ => None,
                //     })
                //     .collect(),
                _ => children
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, child)| match child.kind() {
                        SyntaxKind::Semicolon
                        | SyntaxKind::ObjectListPath
                        | SyntaxKind::ObjectList => Some(SimplifiedTextEdit::new(
                            TextRange::empty(child.text_range().start()),
                            " ",
                        )),
                        _ => None,
                    })
                    .collect(),
            },

            SyntaxKind::BlankNodePropertyListPath => {
                match children.get(1).and_then(|child| child.as_node()) {
                    Some(prop_list) => prop_list
                        .children_with_tokens()
                        .filter(|child| !child.kind().is_trivia())
                        .step_by(3)
                        .skip(1)
                        .map(|child| {
                            SimplifiedTextEdit::new(
                                TextRange::empty(child.text_range().start()),
                                &format!("{}  ", self.get_linebreak(indentation)),
                            )
                        })
                        .collect(),
                    None => Vec::new(),
                }
            }
            SyntaxKind::TriplesSameSubjectPath => {
                let subject = children.first();
                let prop_list = children.last().and_then(|node| node.as_node());
                match (subject, prop_list) {
                    (Some(subject), Some(prop_list))
                        if prop_list.kind() == SyntaxKind::PropertyListPathNotEmpty =>
                    {
                        let insert = match self.settings.align_predicates {
                            true => &format!(
                                "{}",
                                " ".repeat((subject.text_range().len() + TextSize::new(1)).into())
                            ),
                            false => "  ",
                        };
                        prop_list
                            .children_with_tokens()
                            .filter(|child| !child.kind().is_trivia())
                            .step_by(3)
                            .skip(1)
                            .map(|child| {
                                SimplifiedTextEdit::new(
                                    TextRange::empty(child.text_range().start()),
                                    &format!("{}{}", self.get_linebreak(indentation), insert),
                                )
                            })
                            .collect()
                    }
                    _ => vec![],
                }
            }

            SyntaxKind::TriplesBlock
            | SyntaxKind::TriplesTemplate
            | SyntaxKind::ConstructTriples
            | SyntaxKind::Quads
            | SyntaxKind::GroupGraphPatternSub => children
                .iter()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::Dot => Some(SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().start()),
                        " ",
                    )),
                    _ => None,
                })
                .collect(),

            SyntaxKind::ExpressionList | SyntaxKind::ObjectList | SyntaxKind::ObjectListPath => {
                children
                    .iter()
                    .filter_map(|child| match child.kind() {
                        SyntaxKind::Comma => {
                            Some(SimplifiedTextEdit::new(child.text_range(), ", "))
                        }
                        _ => None,
                    })
                    .collect()
            }

            SyntaxKind::DescribeQuery => children
                .iter()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::VAR1 | SyntaxKind::VAR2 | SyntaxKind::IRIREF | SyntaxKind::Star => {
                        Some(SimplifiedTextEdit::new(
                            TextRange::empty(child.text_range().start()),
                            " ",
                        ))
                    }
                    _ => None,
                })
                .collect(),
            SyntaxKind::Modify => children
                .iter()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::IRIREF => Some(vec![SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().start()),
                        " ",
                    )]),
                    SyntaxKind::DeleteClause
                    | SyntaxKind::InsertClause
                    | SyntaxKind::UsingClause
                        if child.prev_sibling_or_token().is_some() =>
                    {
                        Some(vec![SimplifiedTextEdit::new(
                            TextRange::empty(child.text_range().start()),
                            &self.get_linebreak(indentation),
                        )])
                    }
                    SyntaxKind::WHERE => Some(vec![
                        SimplifiedTextEdit::new(
                            TextRange::empty(child.text_range().start()),
                            &self.get_linebreak(indentation),
                        ),
                        SimplifiedTextEdit::new(TextRange::empty(child.text_range().end()), " "),
                    ]),
                    _ => None,
                })
                .flatten()
                .collect(),
            SyntaxKind::Aggregate => children
                .iter()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::Semicolon | SyntaxKind::DISTINCT => Some(SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().end()),
                        " ",
                    )),
                    _ => None,
                })
                .collect(),
            SyntaxKind::Update => children
                .iter()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::Semicolon => Some(SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().start()),
                        " ",
                    )),
                    _ => None,
                })
                .collect(),
            SyntaxKind::ANON => vec![SimplifiedTextEdit::new(node.text_range(), "[]")],
            SyntaxKind::Bind => children
                .iter()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::LParen => Some(vec![SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().start()),
                        " ",
                    )]),
                    SyntaxKind::AS => Some(vec![
                        SimplifiedTextEdit::new(TextRange::empty(child.text_range().start()), " "),
                        SimplifiedTextEdit::new(TextRange::empty(child.text_range().end()), " "),
                    ]),
                    _ => None,
                })
                .flatten()
                .collect(),
            SyntaxKind::INTEGER_POSITIVE
            | SyntaxKind::DECIMAL_POSITIVE
            | SyntaxKind::DOUBLE_POSITIVE
            | SyntaxKind::INTEGER_NEGATIVE
            | SyntaxKind::DECIMAL_NEGATIVE
            | SyntaxKind::DOUBLE_NEGATIVE
                if node
                    .parent()
                    .map_or(false, |parent| parent.prev_sibling().is_some()) =>
            {
                vec![SimplifiedTextEdit::new(
                    TextRange::empty(node.text_range().start() + TextSize::new(1)),
                    " ",
                )]
            }
            _ => Vec::new(),
        }
    }

    fn pre_node_augmentation(
        &self,
        node: &SyntaxElement,
        indentation: u8,
    ) -> Option<SimplifiedTextEdit> {
        let insert = match node.kind() {
            SyntaxKind::ConstructTriples
            | SyntaxKind::SubSelect
            | SyntaxKind::SolutionModifier
            | SyntaxKind::Quads
            | SyntaxKind::DatasetClause
            | SyntaxKind::TriplesBlock
            | SyntaxKind::UNION => Some(self.get_linebreak(indentation)),

            SyntaxKind::GraphPatternNotTriples
                if node
                    .as_node()
                    .and_then(|node| node.first_child())
                    .map_or(false, |child| child.kind() != SyntaxKind::Filter) =>
            {
                Some(self.get_linebreak(indentation))
            }

            SyntaxKind::Filter => match node
                .as_node()
                .and_then(|node| node.parent())
                .and_then(|parent| parent.prev_sibling())
            {
                Some(prev)
                    if prev.kind() == SyntaxKind::TriplesBlock
                        && dbg!(self.text.get(
                            prev.text_range().end().into()..node.text_range().start().into()
                        ))
                        .map_or(false, |s| !s.contains("\n"))
                        && self.settings.filter_same_line =>
                {
                    Some(" ".to_string())
                }
                _ => Some(self.get_linebreak(indentation)),
            },
            SyntaxKind::QuadsNotTriples | SyntaxKind::Update | SyntaxKind::UpdateOne
                if node
                    .as_node()
                    .and_then(|node| node.first_token().unwrap().prev_token())
                    .map_or(false, |prev| prev.kind() != SyntaxKind::Dot) =>
            {
                Some(self.get_linebreak(indentation))
            }
            SyntaxKind::PropertyListPathNotEmpty => match node.parent().map(|parent| parent.kind())
            {
                Some(SyntaxKind::BlankNodePropertyListPath)
                    if node
                        .as_node()
                        .map(|node| node.children_with_tokens().count() > 3)
                        .unwrap_or(false) =>
                {
                    Some(self.get_linebreak(indentation))
                }
                Some(SyntaxKind::BlankNodePropertyListPath)
                    if node
                        .as_node()
                        .map_or(false, |node| node.children_with_tokens().count() <= 3) =>
                {
                    Some(" ".to_string())
                }
                _ => None,
            },
            SyntaxKind::TriplesTemplate => {
                match node
                    .as_node()
                    .and_then(prev_non_comment_sibling)
                    .map(|node| node.kind())
                {
                    Some(x) if x != SyntaxKind::Dot => Some(self.get_linebreak(indentation)),
                    _ => None,
                }
            }
            SyntaxKind::WhereClause => {
                match self.settings.where_new_line
                    || node
                        .parent()
                        .map_or(false, |parent| parent.kind() == SyntaxKind::ConstructQuery)
                    || node
                        .parent()
                        .map(|parent| parent.kind() == SyntaxKind::DescribeQuery)
                        .unwrap_or(false)
                    || node
                        .as_node()
                        .and_then(|node| node.prev_sibling())
                        .map(|sibling| sibling.kind() == SyntaxKind::DatasetClause)
                        .unwrap_or(false)
                {
                    true => Some(self.get_linebreak(indentation)),
                    false => Some(" ".to_string()),
                }
            }
            _ => None,
        }?;
        Some(SimplifiedTextEdit::new(
            TextRange::empty(node.text_range().start()),
            &insert,
        ))
    }

    fn post_node_augmentation(
        &self,
        node: &SyntaxElement,
        indentation: u8,
    ) -> Option<SimplifiedTextEdit> {
        let insert = match node.kind() {
            SyntaxKind::UNION => Some(" ".to_string()),
            SyntaxKind::Prologue
                if self.settings.separate_prolouge
                    && node
                        .as_node()
                        .and_then(|node| node.next_sibling())
                        .is_some() =>
            {
                Some(self.get_linebreak(indentation))
            }
            SyntaxKind::PropertyListPathNotEmpty => match node.parent().map(|parent| parent.kind())
            {
                Some(SyntaxKind::BlankNodePropertyListPath)
                    if node
                        .as_node()
                        .map_or(false, |node| node.children().count() > 3) =>
                {
                    Some(self.get_linebreak(indentation.saturating_sub(1)))
                }
                Some(SyntaxKind::BlankNodePropertyListPath)
                    if node
                        .as_node()
                        .map_or(false, |node| node.children().count() <= 3) =>
                {
                    Some(" ".to_string())
                }
                _ => None,
            },
            // SyntaxKind::TriplesTemplate => match node.parent().map(|parent| parent.kind()) {
            //     Some(SyntaxKind::TriplesTemplateBlock) => {
            //         Some(get_linebreak(&indentation.saturating_sub(1), indent_base))
            //     }
            //     _ => None,
            // },
            SyntaxKind::GroupGraphPatternSub
            | SyntaxKind::ConstructTriples
            | SyntaxKind::Quads
            | SyntaxKind::SubSelect => Some(self.get_linebreak(indentation.saturating_sub(1))),
            _ => None,
        }?;
        Some(SimplifiedTextEdit::new(
            TextRange::empty(node.text_range().end()),
            &insert,
        ))
    }

    #[inline]
    fn get_linebreak(&self, indentation: u8) -> String {
        format!("\n{}", self.indent_base.repeat(indentation as usize))
    }
}

impl Iterator for Walker<'_> {
    type Item = Vec<SimplifiedTextEdit>;

    fn next(&mut self) -> Option<Self::Item> {
        let (element, indentation) = self.queue.pop()?;
        // NOTE: Extract comments
        // let (children, mut comments): (Vec<SyntaxElement>, Vec<CommentMarker>) = element
        //     .as_node()
        //     .map(|node| {
        //         node.children_with_tokens()
        //             .fold((vec![], vec![]), |mut acc, child| {
        //                 match child.kind() {
        //                     SyntaxKind::WHITESPACE => {}
        //                     SyntaxKind::Comment if node.kind() != SyntaxKind::Error => {
        //                         acc.1.push(comment_marker(&child, indentation))
        //                     }
        //                     _ => acc.0.push(child),
        //                 };
        //                 return acc;
        //             })
        //     })
        //     .unwrap_or_default();
        let children: Vec<SyntaxElement> = element
            .as_node()
            .map(|node| {
                node.children_with_tokens()
                    .filter(|child| !child.kind().is_trivia())
                    .collect()
            })
            .unwrap_or_default();

        // NOTE: Step 1: Separation
        let separator = get_separator(element.kind());
        let seperation_edits = children
            .iter()
            .zip(children.iter().skip(1))
            .filter_map(|(node1, node2)| match separator {
                Seperator::LineBreak => Some(SimplifiedTextEdit::new(
                    TextRange::new(node1.text_range().end(), node2.text_range().start()),
                    &self.get_linebreak(indentation),
                )),
                Seperator::Space => Some(SimplifiedTextEdit::new(
                    TextRange::new(node1.text_range().end(), node2.text_range().start()),
                    " ",
                )),
                Seperator::Empty if node2.kind() == SyntaxKind::Error => {
                    Some(SimplifiedTextEdit::new(
                        TextRange::new(node1.text_range().end(), node2.text_range().start()),
                        " ",
                    ))
                }
                Seperator::Empty => Some(SimplifiedTextEdit::new(
                    TextRange::new(node1.text_range().end(), node2.text_range().start()),
                    "",
                )),
                Seperator::Unknown => None,
            })
            .collect::<Vec<_>>();

        // NOTE: Step 2: Augmentation
        let augmentation_edits = self.node_augmentation(&element, &children, indentation);

        if let SyntaxElement::Node(node) = &element {
            self.queue.extend(
                node.children_with_tokens().zip(std::iter::repeat(
                    indentation
                        + INC_INDENTATION
                            .contains(&node.kind())
                            .then(|| 1)
                            .unwrap_or(0),
                )),
            );
        }

        Some(
            augmentation_edits
                .into_iter()
                .chain(seperation_edits.into_iter())
                .collect(),
        )
    }
}

enum Seperator {
    LineBreak,
    Space,
    Empty,
    Unknown,
}

fn get_separator(kind: SyntaxKind) -> Seperator {
    match kind {
        SyntaxKind::QueryUnit
        | SyntaxKind::Query
        | SyntaxKind::Prologue
        | SyntaxKind::SolutionModifier
        | SyntaxKind::LimitOffsetClauses => Seperator::LineBreak,
        SyntaxKind::ExpressionList
        | SyntaxKind::GroupGraphPattern
        | SyntaxKind::GroupGraphPatternSub
        | SyntaxKind::GroupOrUnionGraphPattern
        | SyntaxKind::TriplesTemplate
        | SyntaxKind::BrackettedExpression
        | SyntaxKind::ConstructTemplate
        | SyntaxKind::QuadData
        | SyntaxKind::ObjectList
        | SyntaxKind::ObjectListPath
        | SyntaxKind::SubstringExpression
        | SyntaxKind::RegexExpression
        | SyntaxKind::ArgList
        | SyntaxKind::OrderCondition
        | SyntaxKind::Aggregate
        | SyntaxKind::BuiltInCall
        | SyntaxKind::FunctionCall
        | SyntaxKind::PathSequence
        | SyntaxKind::PathEltOrInverse
        | SyntaxKind::PathElt
        | SyntaxKind::PathPrimary
        | SyntaxKind::PNAME_NS
        | SyntaxKind::BlankNodePropertyListPath
        | SyntaxKind::TriplesBlock
        | SyntaxKind::Quads
        | SyntaxKind::ConstructTriples
        | SyntaxKind::ConstructQuery
        | SyntaxKind::SelectQuery
        | SyntaxKind::SubSelect
        | SyntaxKind::AskQuery
        | SyntaxKind::DescribeQuery
        | SyntaxKind::Modify
        | SyntaxKind::Update
        | SyntaxKind::UpdateOne
        | SyntaxKind::SelectClause
        | SyntaxKind::GroupCondition
        | SyntaxKind::PropertyListPathNotEmpty
        | SyntaxKind::Bind => Seperator::Empty,
        SyntaxKind::BaseDecl
        | SyntaxKind::PrefixDecl
        | SyntaxKind::WhereClause
        | SyntaxKind::DatasetClause
        | SyntaxKind::MinusGraphPattern
        | SyntaxKind::DefaultGraphClause
        | SyntaxKind::NamedGraphClause
        | SyntaxKind::TriplesSameSubject
        | SyntaxKind::OptionalGraphPattern
        | SyntaxKind::ServiceGraphPattern
        | SyntaxKind::InlineData
        | SyntaxKind::InlineDataOneVar
        | SyntaxKind::ValuesClause
        | SyntaxKind::DataBlock
        | SyntaxKind::GroupClause
        | SyntaxKind::HavingClause
        | SyntaxKind::OrderClause
        | SyntaxKind::LimitClause
        | SyntaxKind::OffsetClause
        | SyntaxKind::ExistsFunc
        | SyntaxKind::Filter
        | SyntaxKind::Load
        | SyntaxKind::Clear
        | SyntaxKind::Drop
        | SyntaxKind::Add
        | SyntaxKind::Move
        | SyntaxKind::Copy
        | SyntaxKind::Create
        | SyntaxKind::InsertData
        | SyntaxKind::DeleteData
        | SyntaxKind::DeleteWhere
        | SyntaxKind::GraphRef
        | SyntaxKind::GraphRefAll
        | SyntaxKind::GraphOrDefault
        | SyntaxKind::DeleteClause
        | SyntaxKind::InsertClause
        | SyntaxKind::UsingClause
        | SyntaxKind::PropertyListNotEmpty
        | SyntaxKind::Path
        | SyntaxKind::TriplesSameSubjectPath
        | SyntaxKind::QuadsNotTriples
        | SyntaxKind::PathAlternative
        | SyntaxKind::RelationalExpression
        | SyntaxKind::ConditionalAndExpression
        | SyntaxKind::ConditionalOrExpression
        | SyntaxKind::MultiplicativeExpression
        | SyntaxKind::AdditiveExpression => Seperator::Space,

        _ => Seperator::Unknown,
    }
}

fn prev_non_comment_sibling<'a>(node: &SyntaxNode) -> Option<SyntaxNode> {
    let mut prev = node.prev_sibling()?;
    while matches!(prev.kind(), SyntaxKind::Comment) {
        prev = prev.prev_sibling()?;
    }
    Some(prev)
}

fn transform_edits(simplified_edits: Vec<SimplifiedTextEdit>, text: &str) -> Vec<TextEdit> {
    let mut position = Position::new(0, 0);
    let mut byte_offset = 0;
    let mut marker = position.clone();
    let mut edits = simplified_edits.into_iter().rev();
    let mut result = Vec::new();
    let mut chars = text.chars();
    let mut next_edit = edits.next().unwrap();
    let mut next_char = chars.next();
    loop {
        if TextSize::new(byte_offset) == next_edit.range.start() {
            marker = position.clone();
        }

        if TextSize::new(byte_offset) == next_edit.range.end() {
            result.push(TextEdit::new(
                Range {
                    start: marker,
                    end: position,
                },
                &next_edit.text,
            ));
            next_edit = if let Some(edit) = edits.next() {
                edit
            } else {
                break;
            }
        } else if let Some(char) = next_char {
            byte_offset += char.len_utf8() as u32;
            if matches!(char, '\n') {
                position.line += 1;
                position.character = 0;
            } else {
                position.character += char.len_utf16() as u32;
            }
            next_char = chars.next();
        }
    }
    result
}

fn consolidate_edits(edits: Vec<TextEdit>) -> Vec<ConsolidatedTextEdit> {
    let accumulator: Vec<ConsolidatedTextEdit> = Vec::new();
    edits.into_iter().fold(accumulator, |mut acc, edit| {
        if edit.is_empty() {
            return acc;
        }
        match acc.last_mut() {
            Some(next_consolidated) => match next_consolidated.edits.first_mut() {
                Some(next_edit) if next_edit.range.start == edit.range.end => {
                    next_consolidated.edits.insert(0, edit);
                }
                Some(next_edit)
                    if next_edit.range.start == next_edit.range.end
                        && next_edit.range.start == edit.range.start =>
                {
                    next_edit.new_text.push_str(&edit.new_text);
                    next_edit.range.end = edit.range.end;
                }
                Some(_next_edit) => {
                    acc.push(ConsolidatedTextEdit::new(edit));
                }
                None => {
                    next_consolidated.edits.push(edit);
                }
            },
            None => {
                acc.push(ConsolidatedTextEdit::new(edit));
            }
        };
        acc
    })
}

fn remove_redundent_edits(edits: Vec<TextEdit>, document: &TextDocumentItem) -> Vec<TextEdit> {
    edits
        .into_iter()
        .filter(|edit| {
            if let Some(slice) = document.get_range(&edit.range) {
                if edit.new_text == slice {
                    return false;
                }
            }
            true
        })
        .collect()
}

#[derive(Debug)]
struct ConsolidatedTextEdit {
    edits: Vec<TextEdit>,
}

impl fmt::Display for ConsolidatedTextEdit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self
            .edits
            .iter()
            .map(|edit| edit.to_string())
            .collect::<Vec<_>>()
            .join("|");
        write!(f, "{} = {}", self.fuse(), s)
    }
}

impl ConsolidatedTextEdit {
    fn fuse(&self) -> TextEdit {
        TextEdit::new(
            self.range(),
            &self
                .edits
                .iter()
                .flat_map(|edit| edit.new_text.chars())
                .collect::<String>(),
        )
    }

    fn range(&self) -> Range {
        Range {
            start: self.edits.first().unwrap().range.start,
            end: self.edits.last().unwrap().range.end,
        }
    }

    fn new(edit: TextEdit) -> Self {
        Self { edits: vec![edit] }
    }

    fn split_at(self, position: Position) -> (ConsolidatedTextEdit, ConsolidatedTextEdit) {
        let before = ConsolidatedTextEdit { edits: Vec::new() };
        let after = ConsolidatedTextEdit { edits: Vec::new() };
        self.edits
            .into_iter()
            .fold((before, after), |(mut before, mut after), edit| {
                match (edit.range.start, edit.range.end, position) {
                    (start, end, position) if start < position && position >= end => {
                        before.edits.push(edit)
                    }
                    _ => after.edits.push(edit),
                };
                (before, after)
            })
    }
}

// fn comment_marker(comment_node: &SyntaxElement, indentation: u8) -> CommentMarker {
//     assert_eq!(comment_node.kind(), SyntaxKind::Comment);
//     let mut maybe_attach = Some(*comment_node);
//     while let Some(kind) = maybe_attach.map(|node| node.kind()) {
//         match kind {
//             SyntaxKind::Comment | SyntaxKind::WHITESPACE => {
//                 maybe_attach = maybe_attach.and_then(|node| node.prev_sibling_or_token())
//             }
//             _ => break,
//         }
//     }
//     let attach = maybe_attach
//         .or(comment_node.parent().map(SyntaxElement::Node))
//         .expect("all comment nodes should have a parent");
//     CommentMarker {
//         text: comment_node.to_string(),
//         position: match attach.kind() {
//             SyntaxKind::QueryUnit => Position::new(0, 0),
//             _ => Position::from_point(attach.end_position()),
//         },
//         trailing: attach.end_position().row == comment_node.start_position().row,
//         indentation_level: indentation,
//     }
// }
