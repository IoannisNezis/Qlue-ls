//! Suggests shortening full IRIs that could be written as a prefixed name.
//!
//! Every `iri` node in the query is checked against the server's known
//! namespaces; when a matching prefix exists, an informational hint proposes
//! the compacted CURIE and carries the prefix/namespace/CURIE for a code action.

use std::sync::LazyLock;

use crate::server::{
    Server,
    lsp::{
        base_types::LSPAny,
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        textdocument::{Range, TextDocumentItem},
    },
};
use ll_sparql_parser::ast::{AstNode, Iri, Unit};

pub static CODE: LazyLock<DiagnosticCode> =
    LazyLock::new(|| DiagnosticCode::String("uncompacted-uri".to_string()));

pub(super) fn diagnostics(
    document: &TextDocumentItem,
    ast: &Unit,
    server: &Server,
) -> Option<Vec<Diagnostic>> {
    Some(
        ast.strip_prologue()?
            .descendants()
            .filter_map(Iri::cast)
            .filter_map(|iri| match iri.raw_iri() {
                Some(raw_iri) => match server.shorten_uri(&raw_iri, None) {
                    Some((prefix, namespace, curie)) => Some(Diagnostic {
                        source: None,
                        code: Some((*CODE).clone()),
                        range: Range::from_byte_offset_range(
                            iri.syntax().text_range(),
                            &document.text,
                        )?,
                        severity: DiagnosticSeverity::Information,
                        message: format!(
                            "You might want to shorten this URI\n{} -> {}",
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

#[cfg(test)]
mod test {
    use super::{CODE, diagnostics};
    use crate::server::{
        Server,
        configuration::BackendConfiguration,
        lsp::{base_types::LSPAny, diagnostic::DiagnosticSeverity, textdocument::Range},
        message_handler::diagnostic::test_utils::{run, run_with_server},
    };
    use indoc::indoc;
    use std::collections::HashMap;

    fn setup_server() -> Server {
        let mut server = Server::new(|_message| {});
        server.state.add_backend(BackendConfiguration {
            name: "test".to_string(),
            url: "".to_string(),
            health_check_url: None,
            engine: None,
            request_method: None,
            prefix_map: HashMap::new(),
            default: true,
            queries: HashMap::new(),
            additional_data: None,
        });
        server.state.set_default_backend("test".to_string());
        server.state.load_prefix_map(
            "test".to_string(),
            &HashMap::from_iter([("schema".to_string(), "https://schema.org/".to_string())]),
        );
        server
    }

    #[test]
    fn triggered_by_compactable_iri() {
        let result = run_with_server(
            diagnostics,
            &setup_server(),
            "SELECT * WHERE { ?s <https://schema.org/name> ?o }",
        );
        assert_eq!(result.len(), 1);
        let diagnostic = &result[0];
        assert_eq!(diagnostic.range, Range::new(0, 20, 0, 45));
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Information);
        assert_eq!(diagnostic.code, Some((*CODE).clone()));
        assert_eq!(
            diagnostic.data,
            Some(LSPAny::LSPArray(vec![
                LSPAny::String("schema".to_string()),
                LSPAny::String("https://schema.org/".to_string()),
                LSPAny::String("schema:name".to_string()),
            ]))
        );
    }

    #[test]
    fn not_triggered_by_unknown_namespace() {
        let result = run_with_server(
            diagnostics,
            &setup_server(),
            "SELECT * WHERE { ?s <http://example.org/name> ?o }",
        );
        assert!(result.is_empty());
    }

    #[test]
    fn not_triggered_by_prefixed_name_or_prologue() {
        let result = run_with_server(
            diagnostics,
            &setup_server(),
            indoc!(
                "PREFIX schema: <https://schema.org/>
                 SELECT * WHERE { ?s schema:name ?o }"
            ),
        );
        assert!(result.is_empty());
    }

    #[test]
    fn not_triggered_without_prefix_map() {
        let result = run(
            diagnostics,
            "SELECT * WHERE { ?s <https://schema.org/name> ?o }",
        );
        assert!(result.is_empty());
    }
}
