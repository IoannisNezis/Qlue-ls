use core::fmt;
use std::vec;

use ll_sparql_parser::{parse, syntax_kind::SyntaxKind, SyntaxElement, SyntaxNode};
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
    settings.insert_spaces.unwrap_or(options.insert_spaces);
    let indent_string = match settings.insert_spaces.unwrap_or(options.insert_spaces) {
        true => " ".repeat(settings.tab_size.unwrap_or(options.tab_size) as usize),
        false => "\t".to_string(),
    };
    let walker = Walker::new(
        parse(&document.text),
        &document.text,
        settings,
        indent_string.clone(),
    );

    let (simplified_edits, simpified_comments) = walker.collect_edits_and_comments();
    let comments = transform_comments(simpified_comments, &document.text);
    let mut edits = transform_edits(simplified_edits, &document.text);
    edits.sort_by(|a, b| {
        b.range
            .start
            .cmp(&a.range.start)
            .then_with(|| b.range.end.cmp(&a.range.end))
    });
    let consolidated_edits = consolidate_edits(edits);
    edits = merge_comments(consolidated_edits, comments, &document.text, &indent_string)?;

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
    indentation_level: u8,
    trailing: bool,
}

impl CommentMarker {
    fn to_edit(&self, indent_base: &str) -> TextEdit {
        let prefix = match (
            self.position.line == 0 && self.position.character == 0,
            self.trailing,
        ) {
            (true, _) => "",
            (false, true) => " ",
            (false, false) => &format!("\n{}", indent_base.repeat(self.indentation_level as usize)),
        };

        TextEdit::new(
            Range::new(
                self.position.line,
                self.position.character,
                self.position.line,
                self.position.character,
            ),
            &format!("{}{}", prefix, &self.text),
        )
    }
}

#[derive(Debug)]
struct SimplifiedCommentMarker {
    text: String,
    position: TextSize,
    indentation_level: u8,
    trailing: bool,
}

fn inc_indent(node: &SyntaxNode) -> u8 {
    match node.kind() {
        SyntaxKind::BlankNodePropertyListPath
        | SyntaxKind::GroupGraphPattern
        | SyntaxKind::BrackettedExpression
        | SyntaxKind::ConstructTemplate
        | SyntaxKind::Quads
        | SyntaxKind::QuadsNotTriples => 1,
        SyntaxKind::ConstructQuery
            if node
                .first_child()
                .is_some_and(|node| node.kind() != SyntaxKind::ConstructTemplate) =>
        {
            1
        }
        _ => 0,
    }
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
        children: &[SyntaxElement],
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
        if node.kind() == SyntaxKind::IN {
            if self.settings.capitalize_keywords {
                augmentations.push(SimplifiedTextEdit::new(node.text_range(), "IN"));
            } else {
                augmentations.push(SimplifiedTextEdit::new(node.text_range(), "in"));
            }
        } else if KEYWORDS.contains(&node.kind()) && self.settings.capitalize_keywords {
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
        children: &[SyntaxElement],
        indentation: u8,
    ) -> Vec<SimplifiedTextEdit> {
        match node.kind() {
            SyntaxKind::QueryUnit | SyntaxKind::UpdateUnit => {
                match (children.first(), children.last()) {
                    (Some(first), Some(last)) => vec![
                        SimplifiedTextEdit::new(
                            TextRange::new(0.into(), first.text_range().start()),
                            "",
                        ),
                        SimplifiedTextEdit::new(
                            TextRange::new(last.text_range().end(), node.text_range().end()),
                            "",
                        ),
                    ],
                    _ => vec![SimplifiedTextEdit::new(
                        TextRange::new(0.into(), node.text_range().end()),
                        "",
                    )],
                }
            }
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
                        .is_some_and(|prev| prev.kind() == SyntaxKind::LParen) =>
                    {
                        None
                    }
                    _ if idx > 0
                        && children
                            .get(idx - 1)
                            .is_some_and(|prev| prev.kind() == SyntaxKind::LParen) =>
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

            SyntaxKind::PropertyListPathNotEmpty => children
                .iter()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::Semicolon | SyntaxKind::ObjectListPath | SyntaxKind::ObjectList => {
                        Some(SimplifiedTextEdit::new(
                            TextRange::empty(child.text_range().start()),
                            " ",
                        ))
                    }
                    _ => None,
                })
                .collect(),

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
                let subject = children.first().and_then(|element| element.as_node());
                let prop_list = children.last().and_then(|node| node.as_node());
                match (subject, prop_list) {
                    (Some(subject), Some(prop_list))
                        if prop_list.kind() == SyntaxKind::PropertyListPathNotEmpty =>
                    {
                        let insert = match self.settings.align_predicates {
                            true => &" ".repeat((subject.to_string().chars().count() + 1).into()),
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
                    SyntaxKind::VAR1
                    | SyntaxKind::VAR2
                    | SyntaxKind::VarOrIri
                    | SyntaxKind::Star => Some(SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().start()),
                        " ",
                    )),
                    _ => None,
                })
                .collect(),
            SyntaxKind::Modify => children
                .iter()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::iri => Some(vec![SimplifiedTextEdit::new(
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
                    .is_some_and(|parent| parent.prev_sibling().is_some()) =>
            {
                vec![SimplifiedTextEdit::new(
                    TextRange::empty(node.text_range().start() + TextSize::new(1)),
                    " ",
                )]
            }
            SyntaxKind::DELETE => {
                if self.settings.capitalize_keywords {
                    vec![SimplifiedTextEdit::new(node.text_range(), "DELETE")]
                } else if node.text_range().len() > TextSize::new(6) {
                    vec![SimplifiedTextEdit::new(
                        TextRange::new(TextSize::new(6), node.text_range().end()),
                        "",
                    )]
                } else {
                    vec![]
                }
            }
            SyntaxKind::QuadPattern | SyntaxKind::QuadData => children
                .iter()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::RCurly => Some(SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().start()),
                        "\n",
                    )),
                    _ => None,
                })
                .collect(),
            SyntaxKind::QuadsNotTriples => children
                .iter()
                .filter_map(|child| match child.kind() {
                    SyntaxKind::GRAPH => Some(SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().end()),
                        " ",
                    )),
                    SyntaxKind::VarOrIri => Some(SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().end()),
                        " ",
                    )),
                    SyntaxKind::RCurly => Some(SimplifiedTextEdit::new(
                        TextRange::empty(child.text_range().start()),
                        &self.get_linebreak(indentation),
                    )),
                    _ => None,
                })
                .collect(),
            SyntaxKind::DELETE_WHERE => {
                let start = node.text_range().start();
                let p1 = node.text_range().start() + TextSize::new(6);
                let end = node.text_range().end();
                let p2 = node.text_range().end() - TextSize::new(5);

                let mut res = vec![SimplifiedTextEdit::new(TextRange::new(p1, p2), " ")];
                if self.settings.capitalize_keywords {
                    res.push(SimplifiedTextEdit::new(TextRange::new(start, p1), "DELETE"));
                    res.push(SimplifiedTextEdit::new(TextRange::new(p2, end), "WHERE"));
                }
                res
            }
            SyntaxKind::DELETE_DATA => {
                let start = node.text_range().start();
                let p1 = node.text_range().start() + TextSize::new(6);
                let end = node.text_range().end();
                let p2 = node.text_range().end() - TextSize::new(4);

                let mut res = vec![SimplifiedTextEdit::new(TextRange::new(p1, p2), " ")];
                if self.settings.capitalize_keywords {
                    res.push(SimplifiedTextEdit::new(TextRange::new(start, p1), "DELETE"));
                    res.push(SimplifiedTextEdit::new(TextRange::new(p2, end), "DATA"));
                }
                res
            }
            SyntaxKind::INSERT_DATA => {
                let start = node.text_range().start();
                let p1 = node.text_range().start() + TextSize::new(6);
                let end = node.text_range().end();
                let p2 = node.text_range().end() - TextSize::new(4);

                let mut res = vec![SimplifiedTextEdit::new(TextRange::new(p1, p2), " ")];
                if self.settings.capitalize_keywords {
                    res.push(SimplifiedTextEdit::new(TextRange::new(start, p1), "INSERT"));
                    res.push(SimplifiedTextEdit::new(TextRange::new(p2, end), "DATA"));
                }
                res
            }
            SyntaxKind::a => vec![SimplifiedTextEdit::new(node.text_range(), "a")],
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
            | SyntaxKind::DatasetClause
            | SyntaxKind::TriplesTemplate
            | SyntaxKind::TriplesBlock
            | SyntaxKind::UNION => Some(self.get_linebreak(indentation)),

            SyntaxKind::GraphPatternNotTriples
                if node
                    .as_node()
                    .and_then(|node| node.first_child())
                    .is_some_and(|child| child.kind() != SyntaxKind::Filter) =>
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
                        && self
                            .text
                            .get(prev.text_range().end().into()..node.text_range().start().into())
                            .is_some_and(|s| !s.contains("\n"))
                        && self.settings.filter_same_line =>
                {
                    Some(" ".to_string())
                }
                _ => Some(self.get_linebreak(indentation)),
            },
            SyntaxKind::QuadsNotTriples | SyntaxKind::UpdateOne
                if node
                    .as_node()
                    .and_then(|node| node.first_token().unwrap().prev_token())
                    .is_some_and(|prev| prev.kind() != SyntaxKind::Dot) =>
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
                        .is_some_and(|node| node.children_with_tokens().count() <= 3) =>
                {
                    Some(" ".to_string())
                }
                _ => None,
            },
            SyntaxKind::WhereClause => {
                match self.settings.where_new_line
                    || node
                        .parent()
                        .is_some_and(|parent| parent.kind() == SyntaxKind::ConstructQuery)
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
                if self.settings.separate_prologue
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
                        .is_some_and(|node| node.children().count() > 3) =>
                {
                    Some(self.get_linebreak(indentation.saturating_sub(1)))
                }
                Some(SyntaxKind::BlankNodePropertyListPath)
                    if node
                        .as_node()
                        .is_some_and(|node| node.children().count() <= 3) =>
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

    fn comment_marker(
        &self,
        comment_node: &SyntaxElement,
        indentation_level: u8,
    ) -> SimplifiedCommentMarker {
        assert_eq!(comment_node.kind(), SyntaxKind::Comment);
        let mut maybe_attach = Some(comment_node.clone());
        while let Some(kind) = maybe_attach.as_ref().map(|node| node.kind()) {
            if !kind.is_trivia() {
                break;
            }
            maybe_attach = maybe_attach.and_then(|node| node.prev_sibling_or_token())
        }
        let attach = maybe_attach
            .or(comment_node.parent().map(SyntaxElement::Node))
            .expect("all comment nodes should have a parent");

        let trailing = self
            .text
            .get(attach.text_range().end().into()..comment_node.text_range().start().into())
            .is_some_and(|s| !s.contains("\n"));
        SimplifiedCommentMarker {
            text: comment_node.to_string(),
            position: match attach.kind() {
                SyntaxKind::QueryUnit | SyntaxKind::UpdateUnit => TextSize::new(0),
                _ => attach.text_range().end(),
            },

            trailing,
            indentation_level,
        }
    }

    fn collect_edits_and_comments(self) -> (Vec<SimplifiedTextEdit>, Vec<SimplifiedCommentMarker>) {
        let mut res_edits = Vec::new();
        let mut res_comments = Vec::new();
        for (edits, comments) in self.into_iter() {
            res_edits.extend(edits);
            res_comments.extend(comments);
        }
        (res_edits, res_comments)
    }
}

impl Iterator for Walker<'_> {
    type Item = (Vec<SimplifiedTextEdit>, Vec<SimplifiedCommentMarker>);

    fn next(&mut self) -> Option<Self::Item> {
        let (element, indentation) = self.queue.pop()?;
        // NOTE: Extract comments
        let (children, comments): (Vec<SyntaxElement>, Vec<SimplifiedCommentMarker>) = element
            .as_node()
            .map(|node| {
                node.children_with_tokens()
                    .fold((vec![], vec![]), |mut acc, child| {
                        match child.kind() {
                            SyntaxKind::WHITESPACE => {}
                            SyntaxKind::Comment if node.kind() != SyntaxKind::Error => acc
                                .1
                                .push(self.comment_marker(&child, indentation + inc_indent(node))),
                            _ => acc.0.push(child),
                        };
                        acc
                    })
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
            let new_indent = indentation + inc_indent(node);
            self.queue.extend(
                node.children_with_tokens()
                    .zip(std::iter::repeat(new_indent)),
            );
        }

        Some((
            augmentation_edits
                .into_iter()
                .chain(seperation_edits)
                .collect(),
            comments,
        ))
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
        | SyntaxKind::QuadPattern
        | SyntaxKind::QuadsNotTriples
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
        | SyntaxKind::PathAlternative
        | SyntaxKind::RelationalExpression
        | SyntaxKind::ConditionalAndExpression
        | SyntaxKind::ConditionalOrExpression
        | SyntaxKind::MultiplicativeExpression
        | SyntaxKind::AdditiveExpression => Seperator::Space,

        _ => Seperator::Unknown,
    }
}

fn transform_comments(
    mut comments: Vec<SimplifiedCommentMarker>,
    text: &str,
) -> Vec<CommentMarker> {
    if comments.is_empty() {
        return vec![];
    }
    comments.sort_by(|a, b| a.position.cmp(&b.position));
    let mut position = Position::new(0, 0);
    let mut byte_offset = 0;
    let mut comments = comments.into_iter();
    let mut result = Vec::new();
    let mut chars = text.chars();
    let mut next_comment = comments
        .next()
        .expect("There should be atleast one comment, since the length is > 0");
    let mut next_char = chars.next();
    loop {
        if TextSize::new(byte_offset) == next_comment.position {
            result.push(CommentMarker {
                text: next_comment.text,
                position,
                indentation_level: next_comment.indentation_level,
                trailing: next_comment.trailing,
            });
            next_comment = if let Some(comment) = comments.next() {
                comment
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

fn transform_edits(mut simplified_edits: Vec<SimplifiedTextEdit>, text: &str) -> Vec<TextEdit> {
    simplified_edits.sort_by(|a, b| {
        b.range
            .end()
            .cmp(&a.range.end())
            .then_with(|| b.range.start().cmp(&a.range.start()))
    });
    let mut position = Position::new(0, 0);
    let mut byte_offset = 0;
    let mut marker = position;
    let mut edits = simplified_edits.into_iter().rev();
    let mut result = Vec::new();
    let mut chars = text.chars();
    let mut next_edit = edits.next().unwrap();
    let mut next_char = chars.next();
    loop {
        if TextSize::new(byte_offset) == next_edit.range.start() {
            marker = position;
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
            start: self
                .edits
                .first()
                .expect("There should always be atleast one edit")
                .range
                .start,
            end: self
                .edits
                .last()
                .expect("There should always be atleast one edit")
                .range
                .end,
        }
    }

    fn new(edit: TextEdit) -> Self {
        Self { edits: vec![edit] }
    }

    fn split_at(self, position: Position) -> (ConsolidatedTextEdit, ConsolidatedTextEdit) {
        let before = ConsolidatedTextEdit { edits: Vec::new() };
        let after = ConsolidatedTextEdit { edits: Vec::new() };
        let (before, after) =
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
                });
        (before, after)
    }
}

fn merge_comments(
    edits: Vec<ConsolidatedTextEdit>,
    comments: Vec<CommentMarker>,
    text: &str,
    indent_base: &str,
) -> Result<Vec<TextEdit>, LSPError> {
    let mut comment_iter = comments.into_iter().rev().peekable();
    let mut merged_edits =
        edits
            .into_iter()
            .fold(vec![], |mut acc: Vec<TextEdit>, mut consolidated_edit| {
                let start_position = consolidated_edit.range().start;

                while comment_iter
                    .peek()
                    .map(|comment| comment.position >= start_position)
                    .unwrap_or(false)
                {
                    let mut comment = comment_iter
                        .next()
                        .expect("comment itterator should not be empty");

                    // NOTE: In some Edgecase the comment is in the middle of a (consolidated)
                    // edit. For Example
                    // Select #comment
                    // * {}
                    // In this case this edits needs to be split into two edits.
                    let (previous_edit, next_edit) = consolidated_edit.split_at(comment.position);
                    let (mut previous_edit, mut next_edit) = (previous_edit, next_edit.fuse());
                    // WARNING: This could cause issues.
                    // The amout of chars is neccesarily equal to the amout of
                    // utf-8 bytes. Here i assume that all whispace is 1 utf8 byte long.
                    match next_edit
                        .new_text
                        .chars()
                        .enumerate()
                        .find_map(|(idx, char)| {
                            (!char.is_whitespace() || char == '\n').then_some((idx, char))
                        }) {
                        Some((idx, '\n')) => {
                            next_edit.new_text = format!(
                                "{}{}",
                                comment.to_edit(indent_base).new_text,
                                &next_edit.new_text[idx..]
                            )
                        }
                        Some((idx, _char)) => {
                            next_edit.new_text = format!(
                                "{}\n{}{}",
                                comment.to_edit(indent_base).new_text,
                                indent_base.repeat(comment.indentation_level as usize),
                                &next_edit.new_text[idx..]
                            )
                        }
                        None => {
                            let indent = match next_edit.range.end.byte_index(text) {
                                Some(start_next_token) => {
                                    match text.get(
                                        Into::<usize>::into(start_next_token)
                                            ..Into::<usize>::into(
                                                start_next_token + TextSize::new(1),
                                            ),
                                    ) {
                                        Some("}") => comment.indentation_level.saturating_sub(1),
                                        _ => comment.indentation_level,
                                    }
                                }
                                None => comment.indentation_level,
                            };
                            next_edit.new_text = format!(
                                "{}\n{}",
                                comment.to_edit(indent_base).new_text,
                                indent_base.repeat(indent as usize)
                            )
                        }
                    };
                    previous_edit.edits.push(next_edit);
                    consolidated_edit = previous_edit;
                }

                acc.push(consolidated_edit.fuse());
                acc
            });
    // NOTE: all remaining comments are attached to 0:0.
    comment_iter.for_each(|comment| {
        let comment_edit = comment.to_edit(indent_base);
        merged_edits.push(TextEdit::new(Range::new(0, 0, 0, 0), "\n"));
        merged_edits.push(comment_edit);
    });

    Ok(merged_edits)
}
