use ll_sparql_parser::ast::{AstNode, Iri, PrefixDeclaration, Prologue, QueryUnit};

use super::{
    lsp::errors::{ErrorCode, LSPError},
    state::ServerState,
    Server,
};

pub fn namespace_is_declared(
    server_state: &mut ServerState,
    document_uri: &str,
    namespace: &str,
) -> Result<bool, LSPError> {
    Ok(find_all_prefix_declarations(server_state, document_uri)?
        .iter()
        .any(|node| node.prefix().is_some_and(|prefix| prefix == namespace)))
}

pub fn find_all_uncompacted_iris(
    server: &mut Server,
    document_uri: &str,
) -> Result<Vec<Iri>, LSPError> {
    let root = server.state.get_cached_parse_tree(document_uri)?;
    let query_unit = QueryUnit::cast(root).ok_or(LSPError::new(
        ErrorCode::InternalError,
        "find_all_uncompacted_uris is not jet supported for update",
    ))?;
    Ok(query_unit
        .prologue()
        .and_then(|prologue| prologue.syntax().next_sibling())
        .or(Some(query_unit.syntax().clone()))
        .map(|node| {
            node.descendants()
                .filter_map(Iri::cast)
                .filter(|iri| iri.is_uncompressed())
                .collect()
        })
        .unwrap_or_default())
}

/// Extracts the declared namespaces from a SPARQL document.
///
/// This function parses the specified document to identify namespace declarations
/// (`PrefixDecl`) and returns a list of nodes, each containing the range namespace prefix
///
/// # Arguments
///
/// * `server_state` - A reference to the `ServerState` object, which provides access
///   to the document
/// * `document_uri` - A string slice representing the URI of the document to analyze.
///
/// # Returns
///
/// * `Ok(Vec<PrefixDeclaration>)` - A Vec of nodes in the abstract syntax tree
/// * `Err(LSPError)` - An error if the document
///
/// # Errors
///
/// This function can return a `LSPError` if:
/// * The document specified by `document_uri` cannot be found or loaded.
pub(crate) fn find_all_prefix_declarations(
    server_state: &mut ServerState,
    document_uri: &str,
) -> Result<Vec<PrefixDeclaration>, LSPError> {
    let root = server_state.get_cached_parse_tree(document_uri)?;
    Ok(root
        .first_child()
        .and_then(|child| child.first_child())
        .and_then(Prologue::cast)
        .map(|prologue| prologue.prefix_declarations())
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::server::{
        analysis::{find_all_prefix_declarations, find_all_uncompacted_iris},
        lsp::textdocument::TextDocumentItem,
        state::ServerState,
        Server,
    };

    fn setup_state(text: &str) -> ServerState {
        let mut state = ServerState::new();
        let document = TextDocumentItem::new("uri", text);
        state.add_document(document);
        state
    }

    #[test]
    fn test_find_all_uncompacted_iris() {
        let mut server = Server::new(|_message| {});
        let state = setup_state(indoc!(
            "SELECT * {
               ?a <https://schema.org/name> ?b .
               ?c <https://schema.org/name> ?d
             }"
        ));
        server.state = state;
        let uncompacted_iris = find_all_uncompacted_iris(&mut server, "uri").unwrap();
        assert_eq!(
            uncompacted_iris
                .into_iter()
                .map(|iri| iri.raw_iri().unwrap())
                .collect::<Vec<_>>(),
            vec!["https://schema.org/name", "https://schema.org/name"]
        );
    }

    #[test]
    fn declared_namespaces() {
        let mut state = setup_state(indoc!(
            "PREFIX wdt: <iri>
                 PREFIX wd: <iri>
                 PREFIX wdt: <iri>

                 SELECT * {}"
        ));
        let declared_namespaces = find_all_prefix_declarations(&mut state, "uri").unwrap();
        assert_eq!(
            declared_namespaces
                .iter()
                .filter_map(|prefix_declaration| prefix_declaration.prefix())
                .collect::<Vec<_>>(),
            vec!["wdt", "wd", "wdt"]
        );
    }
}
