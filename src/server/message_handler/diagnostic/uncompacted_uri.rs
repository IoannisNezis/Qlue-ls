use ll_sparql_parser::{
    ast::{AstNode, QueryUnit},
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
            .filter_map(|iri| {
                log::info!("{:?}", iri);
                let iri_string = iri.text().to_string();
                // TODO: Check if iri is a IRIREF and not a PrefixedName
                match server.shorten_uri(&iri_string[1..iri_string.len() - 1]) {
                    Some((prefix, namespace, curie)) => Some(Diagnostic {
                        source: None,
                        code: Some(DiagnosticCode::String("uncompacted-uri".to_string())),
                        range: Range::from_byte_offset_range(iri.text_range(), &document.text)?,
                        severity: DiagnosticSeverity::Information,
                        message: format!(
                            "You might want to shorten this Uri\n{} -> {}",
                            iri_string, curie
                        ),
                        data: Some(LSPAny::LSPArray(vec![
                            LSPAny::String(prefix),
                            LSPAny::String(namespace),
                            LSPAny::String(curie),
                        ])),
                    }),
                    None => None,
                }
            })
            .collect(),
    )
}
