use std::collections::HashMap;

use ll_sparql_parser::{
    ast::{AstNode, SelectQuery},
    SyntaxElement,
};
use text_size::{TextRange, TextSize};

use crate::server::lsp::{
    textdocument::{Position, Range, TextDocumentItem, TextEdit},
    CodeAction, WorkspaceEdit,
};

pub(super) fn code_actions(
    token: SyntaxElement,
    document: &TextDocumentItem,
    indent: u8,
) -> Option<CodeAction> {
    let select_query = match token {
        SyntaxElement::Node(node) => SelectQuery::cast(node),
        SyntaxElement::Token(token) => token.parent_ancestors().nth(1).and_then(SelectQuery::cast),
    }?;
    let select_clause = select_query.select_clause()?;

    let selection = if select_clause.is_star_selection() {
        "*".to_string()
    } else {
        select_clause
            .projected_variables()
            .into_iter()
            .map(|var| var.text())
            .collect::<Vec<_>>()
            .join(" ")
    };
    let indent = " ".repeat(indent as usize);
    let prefix = format!(
        "SELECT {} WHERE {{\n{}{{\n{}",
        selection,
        indent,
        indent.repeat(2)
    );
    let suffix = format!("\n{}}}\n}}", indent);
    let text = select_query.text();
    let indent_edits = text.char_indices().filter_map(|(idx, char)| {
        (char == '\n').then(|| {
            TextEdit::new(
                Range::from_byte_offset_range(
                    TextRange::empty(
                        select_clause.syntax().text_range().start()
                            + TextSize::new((idx + 1) as u32),
                    ),
                    &document.text,
                )
                .unwrap(),
                &indent.repeat(2),
            )
        })
    });
    let mut edits = Vec::new();
    edits.push(TextEdit::new(
        Range::empty(Position::from_byte_index(
            select_query.syntax().text_range().end().into(),
            &document.text,
        )?),
        &suffix,
    ));
    edits.extend(indent_edits);
    edits.push(TextEdit::new(
        Range::empty(Position::from_byte_index(
            select_query.syntax().text_range().start().into(),
            &document.text,
        )?),
        &prefix,
    ));

    Some(CodeAction {
        title: "Transform into Sub-Select".to_string(),
        kind: Some(crate::server::lsp::CodeActionKind::Refactor),
        diagnostics: vec![],
        edit: WorkspaceEdit {
            changes: Some(HashMap::from_iter([(document.uri.to_string(), edits)])),
        },
    })
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use ll_sparql_parser::parse;
    use text_size::{TextRange, TextSize};

    use crate::server::{
        lsp::textdocument::TextDocumentItem, message_handler::code_action::select::code_actions,
    };

    #[test]
    fn transform_into_sub_select() {
        let input = indoc! {
            "SELECT ?var1 (?var2 as ?var3) WHERE {
               ?var1 <> ?var2
             }
             "
        };
        let mut document = TextDocumentItem::new("query.rq", input);
        let tree = parse(&document.text);
        let token = tree.covering_element(TextRange::new(TextSize::new(0), TextSize::new(0)));
        let action = code_actions(token, &document, 2).unwrap();
        document.apply_text_edits(
            action
                .edit
                .changes
                .unwrap()
                .into_values()
                .flatten()
                .collect(),
        );
        assert_eq!(
            document.text,
            indoc! {
                "SELECT ?var1 ?var3 WHERE {
                   {
                     SELECT ?var1 (?var2 as ?var3) WHERE {
                       ?var1 <> ?var2
                     }
                   }
                 }
                 "
            }
        );
    }

    #[test]
    fn transform_into_sub_select_star() {
        let input = indoc! {
            "SELECT * WHERE {
               ?var1 <> ?var2
             }
             "
        };
        let mut document = TextDocumentItem::new("query.rq", input);
        let tree = parse(&document.text);
        let token = tree.covering_element(TextRange::new(TextSize::new(0), TextSize::new(0)));
        let action = code_actions(token, &document, 2).unwrap();
        document.apply_text_edits(
            action
                .edit
                .changes
                .unwrap()
                .into_values()
                .flatten()
                .collect(),
        );
        assert_eq!(
            document.text,
            indoc! {
                "SELECT * WHERE {
                   {
                     SELECT * WHERE {
                       ?var1 <> ?var2
                     }
                   }
                 }
                 "
            }
        );
    }

    #[test]
    fn transform_into_sub_select_star_emogi() {
        let input = indoc! {
            "SELECT ?😀 ?🛰️  {
               ?🛰️ 🌠:P31 🌌:Q1049294 ;
                   🌠:P487 ?😀 .
             }"
        };
        let mut document = TextDocumentItem::new("query.rq", input);
        let tree = parse(&document.text);
        let token = tree.covering_element(TextRange::new(TextSize::new(0), TextSize::new(0)));
        let action = code_actions(token, &document, 2).unwrap();
        document.apply_text_edits(
            action
                .edit
                .changes
                .unwrap()
                .into_values()
                .flatten()
                .collect(),
        );
        assert_eq!(
            document.text,
            indoc! {
                "SELECT ?😀 ?🛰️ WHERE {
                   {
                     SELECT ?😀 ?🛰️  {
                       ?🛰️ 🌠:P31 🌌:Q1049294 ;
                           🌠:P487 ?😀 .
                     }
                   }
                 }
                 "
            }
        );
    }
}
