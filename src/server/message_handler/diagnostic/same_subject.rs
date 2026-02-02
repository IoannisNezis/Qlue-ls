use std::{collections::HashMap, sync::LazyLock};

use crate::server::{
    Server,
    lsp::{
        base_types::LSPAny,
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
};
use ll_sparql_parser::ast::{AstNode, GroupGraphPattern, QueryUnit, Triple};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("same-subject".to_string()));

pub(crate) fn diagnostics(
    document: &TextDocumentItem,
    query_unit: &QueryUnit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    Some(
        query_unit
            .syntax()
            .descendants()
            .filter_map(GroupGraphPattern::cast)
            .flat_map(|ggp| {
                let triples: Vec<_> = ggp
                    .triple_blocks()
                    .into_iter()
                    .flat_map(|tb| tb.triples())
                    .collect();
                let mut groups: HashMap<String, Vec<Triple>> = HashMap::new();
                for triple in triples.into_iter().filter(|triple| !triple.has_error()) {
                    if let Some(subject) = triple.subject() {
                        groups
                            .entry(subject.text())
                            .and_modify(|bucket| bucket.push(triple.clone()))
                            .or_insert(vec![triple]);
                    }
                }
                groups
                    .into_values()
                    .filter_map(|group| {
                        (group.len() >= 2).then_some({
                            let ranges = LSPAny::LSPArray(
                                group
                                    .iter()
                                    .map(|triple| {
                                        let range = triple.syntax().text_range();
                                        LSPAny::LSPObject(HashMap::from_iter([
                                            (
                                                "start".to_string(),
                                                LSPAny::Uinteger(range.start().into()),
                                            ),
                                            (
                                                "end".to_string(),
                                                LSPAny::Uinteger(range.end().into()),
                                            ),
                                        ]))
                                    })
                                    .collect(),
                            );
                            group.into_iter().map(move |triple| Diagnostic {
                                range: Range::from_byte_offset_range(
                                    triple.syntax().text_range(),
                                    &document.text,
                                )
                                .expect("triple text range should be in text"),
                                severity: DiagnosticSeverity::Information,
                                code: Some((*CODE).clone()),
                                source: None,
                                message: format!(
                                    "Triple with same subject \"{}\" can be contracted",
                                    triple.subject().unwrap().text()
                                ),
                                data: Some(ranges.clone()),
                            })
                        })
                    })
                    .flatten()
            })
            .collect(),
    )
}
