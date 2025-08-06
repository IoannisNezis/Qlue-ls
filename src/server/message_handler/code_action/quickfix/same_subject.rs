use ll_sparql_parser::{
    ast::{AstNode, Triple},
    parse,
    syntax_kind::SyntaxKind,
};
use text_size::{TextRange, TextSize};

use crate::server::{
    lsp::{
        base_types::LSPAny,
        diagnostic::Diagnostic,
        errors::{ErrorCode, LSPError},
        textdocument::{Position, Range, TextEdit},
        CodeAction,
    },
    Server,
};

type SameSubjectData = Vec<TextRange>;

pub(crate) fn contract_triples(
    server: &Server,
    document_uri: &String,
    diagnostic: Diagnostic,
) -> Result<Option<CodeAction>, LSPError> {
    let data = extract_data(&diagnostic).ok_or(LSPError::new(
        ErrorCode::InvalidParams,
        "The same-subject diagnostic should have a array of ranges as data",
    ))?;
    let document = server.state.get_document(&document_uri)?;
    let root = parse(&document.text);
    let mut triples = Vec::new();
    for range in data {
        if !root.text_range().contains_range(range) {
            return Err(LSPError::new(
                ErrorCode::InvalidParams,
                &format!("The range {:?} is not contained in the document.", range),
            ));
        }
        let node = root
            .covering_element(range)
            .into_node()
            .ok_or(LSPError::new(
                ErrorCode::InvalidParams,
                &format!("The range {:?} does cover a token and not a node", range),
            ))?;
        let triple = Triple::cast(node).ok_or(LSPError::new(
            ErrorCode::InvalidParams,
            &format!("The range {:?} does not cover a triple", range),
        ))?;
        if triple.has_error() {
            return Err(LSPError::new(
                ErrorCode::InvalidParams,
                &format!("The range {:?} covers a triple that contains errors", range),
            ));
        }
        triples.push(triple);
    }
    let mut code_action = CodeAction::new(
        "contract triples with same subject",
        Some(crate::server::lsp::CodeActionKind::QuickFix),
    );
    for triple in triples.iter().skip(1) {
        let range = triple.syntax().text_range();
        let start = triple
            .syntax()
            .parent()
            .and_then(|parent| parent.prev_sibling_or_token())
            .and_then(|prev| {
                (prev.kind() == SyntaxKind::WHITESPACE).then_some(prev.text_range().start())
            })
            .unwrap_or(range.start());
        let end = triple
            .syntax()
            .next_sibling_or_token_by_kind(&|kind| {
                matches!(kind, SyntaxKind::WHITESPACE | SyntaxKind::Dot)
            })
            .map(|next| {
                next.next_sibling_or_token()
                    .and_then(|next_next| {
                        (next_next.kind() == SyntaxKind::Dot).then_some(next_next)
                    })
                    .unwrap_or(next)
            })
            .map(|next| next.text_range().end())
            .unwrap_or(range.end());
        code_action.add_edit(
            document_uri,
            TextEdit::new(
                Range::from_byte_offset_range(TextRange::new(start, end), &document.text).unwrap(),
                "",
            ),
        );
    }
    if let Some(triple) = triples.first() {
        let verb_position = Position::from_byte_index(
            TextSize::new(
                triple
                    .properties_list_path()
                    .unwrap()
                    .syntax()
                    .to_string()
                    .chars()
                    .count() as u32,
            ),
            &document.text,
        )
        .unwrap();
        let indent_string = " ".repeat(verb_position.character as usize);
        code_action.add_edit(
            document_uri,
            TextEdit::new(
                Range::empty(
                    Position::from_byte_index(triple.syntax().text_range().end(), &document.text)
                        .unwrap(),
                ),
                &format!(
                    " ;\n{}{}",
                    indent_string,
                    triples
                        .iter()
                        .skip(1)
                        .map(|triple| triple.properties_list_path().unwrap().text())
                        .collect::<Vec<_>>()
                        .join(&format!(" ;\n{}", indent_string))
                ),
            ),
        );
    }
    Ok(Some(code_action))
}

fn extract_data(diagnostic: &Diagnostic) -> Option<SameSubjectData> {
    let data = diagnostic.data.as_ref()?;
    if let LSPAny::LSPArray(ranges) = data {
        let mut res = Vec::new();
        for range in ranges.iter() {
            if let LSPAny::LSPObject(map) = range {
                let start = map.get("start");
                let end = map.get("end");
                if let (Some(LSPAny::Uinteger(start)), Some(LSPAny::Uinteger(end))) = (start, end) {
                    res.push(TextRange::new(TextSize::new(*start), TextSize::new(*end)));
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        return Some(res);
    } else {
        None
    }
}
