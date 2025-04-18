use ll_sparql_parser::{
    ast::{AstNode, Iri, QueryUnit},
    syntax_kind::SyntaxKind,
    SyntaxNode,
};

use crate::server::{
    lsp::{
        base_types::LSPAny,
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
    Server,
};

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    server: &Server,
    parse_tree: SyntaxNode,
) -> Option<Vec<Diagnostic>> {
    Some(
        QueryUnit::cast(parse_tree)?
            .select_query()?
            .preorder_find_kind(SyntaxKind::iri)
            .into_iter()
            .filter_map(Iri::cast)
            .filter_map(|iri| match iri.raw_iri() {
                Some(raw_iri) => match server.shorten_uri(&raw_iri, None) {
                    Some((prefix, namespace, curie)) => Some(Diagnostic {
                        source: None,
                        code: Some(DiagnosticCode::String("uncompacted-uri".to_string())),
                        range: Range::from_byte_offset_range(
                            iri.syntax().text_range(),
                            &document.text,
                        )?,
                        severity: DiagnosticSeverity::Information,
                        message: format!(
                            "You might want to shorten this Uri\n{} -> {}",
                            raw_iri, curie
                        ),
                        data: Some(LSPAny::LSPArray(vec![
                            LSPAny::String(prefix),
                            LSPAny::String(namespace),
                            LSPAny::String(curie),
                        ])),
                    }),
                    None => None,
                },
                None => None,
            })
            .collect(),
    )
}
