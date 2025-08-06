use std::{collections::HashMap, sync::LazyLock};

use crate::server::{
    lsp::{
        base_types::LSPAny,
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
    Server,
};
use ll_sparql_parser::ast::{AstNode, GroupGraphPattern, QueryUnit, Triple};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("same-subject".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    query_unit: &QueryUnit,
    _server: &Server,
) -> Option<Vec<Diagnostic>> {
    Some(
        query_unit
            .syntax()
            .descendants()
            .filter_map(GroupGraphPattern::cast)
            .map(|ggp| {
                let triples: Vec<_> = ggp
                    .triple_blocks()
                    .into_iter()
                    .map(|tb| tb.triples())
                    .flatten()
                    .collect();
                let mut buckets: HashMap<String, Vec<Triple>> = HashMap::new();
                for triple in triples.into_iter().filter(|triple| !triple.has_error()) {
                    if let Some(subject) = triple.subject() {
                        buckets
                            .entry(subject.text())
                            .and_modify(|bucket| bucket.push(triple.clone()))
                            .or_insert(vec![triple]);
                    }
                }
                buckets
                    .into_values()
                    .filter(|bucket| bucket.len() >= 2)
                    .map(|bucket| {
                        let ranges = LSPAny::LSPArray(
                            bucket
                                .iter()
                                .map(|triple| {
                                    let range = triple.syntax().text_range();
                                    LSPAny::LSPObject(HashMap::from_iter([
                                        (
                                            "start".to_string(),
                                            LSPAny::Uinteger(range.start().into()),
                                        ),
                                        ("end".to_string(), LSPAny::Uinteger(range.end().into())),
                                    ]))
                                })
                                .collect(),
                        );
                        bucket.into_iter().map(move |triple| Diagnostic {
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
                    .flatten()
            })
            .flatten()
            .collect(),
    )
}
