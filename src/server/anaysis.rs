use super::{
    lsp::{
        errors::{ErrorCode, LSPError},
        textdocument::Range,
    },
    state::ServerState,
    Server,
};
use std::collections::HashSet;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Query, QueryCursor};

fn build_query(query_str: &str) -> Result<Query, LSPError> {
    Query::new(&tree_sitter_sparql::LANGUAGE.into(), query_str).map_err(|error| {
        LSPError::new(
            ErrorCode::InternalError,
            &format!(
                "Building tree-sitter query failed:\n{}\n{}",
                query_str, error
            ),
        )
    })
}

fn collect_all_unique_captures(
    node: Node,
    query_str: &str,
    text: &str,
) -> Result<Vec<String>, LSPError> {
    let query = build_query(query_str)?;
    let mut capture_set: HashSet<String> = HashSet::new();
    let mut query_cursor = QueryCursor::new();
    let mut captures = query_cursor.captures(&query, node, text.as_bytes());
    while let Some((mat, capture_index)) = captures.next() {
        let node: Node = mat.captures[*capture_index].node;
        if node.end_byte() != node.start_byte() {
            capture_set.insert(node.utf8_text(text.as_bytes()).unwrap().to_string());
        }
    }
    Ok(capture_set.into_iter().collect())
}

pub fn namespace_is_declared(
    server_state: &ServerState,
    document_uri: &str,
    namespace: &str,
) -> Result<bool, LSPError> {
    let declared_namespaces: HashSet<String> = get_declared_prefixes(server_state, document_uri)?
        .into_iter()
        .map(|(namespace, _range)| namespace)
        .collect();
    Ok(declared_namespaces.contains(namespace))
}

pub fn get_all_uncompacted_uris(
    server: &Server,
    document_uri: &str,
) -> Result<Vec<(String, Range)>, LSPError> {
    let (document, tree) = server.state.get_state(document_uri)?;
    let declared_uris = collect_all_unique_captures(
        tree.root_node(),
        "(PrefixDecl (IRIREF) @variable)",
        &document.text,
    )?;
    let prefix_set: HashSet<String> = HashSet::from_iter(declared_uris);
    let all_uris = get_all_uris(&server.state, document_uri)?;
    Ok(all_uris
        .into_iter()
        .filter(|(uri, _range)| !prefix_set.contains(uri))
        .collect())
}

fn get_all_uris(
    analyis_state: &ServerState,
    document_uri: &str,
) -> Result<Vec<(String, Range)>, LSPError> {
    let (document, tree) = analyis_state.get_state(document_uri)?;
    let query_str = "(IRIREF) @iri";
    let query = build_query(query_str)?;
    let mut query_cursor = QueryCursor::new();
    let mut captures = query_cursor.captures(&query, tree.root_node(), document.text.as_bytes());
    let mut namespaces: Vec<(String, Range)> = Vec::new();
    while let Some((mat, capture_index)) = captures.next() {
        let node = mat.captures[*capture_index].node;
        namespaces.push((
            node.utf8_text(document.text.as_bytes())
                .unwrap()
                .to_string(),
            Range::from_node(&node),
        ));
    }
    Ok(namespaces)
}

/// Extracts the declared namespaces from a SPARQL document.
///
/// This function parses the specified document to identify namespace declarations
/// (`PrefixDecl`) and returns a list of tuples, each containing the namespace prefix
/// and its corresponding range within the document.
///
/// # Arguments
///
/// * `analysis_state` - A reference to the `ServerState` object, which provides access
///   to the document and its syntax tree.
/// * `document_uri` - A string slice representing the URI of the document to analyze.
///
/// # Returns
///
/// * `Ok(Vec<(String, Range)>)` - A vector of tuples where each tuple consists of:
///   - A `String` representing the namespace prefix.
///   - A `Range` specifying the location of the prefix in the document.
/// * `Err(LSPError)` - An error if the document or its syntax tree cannot be
///   retrieved, or if the query for namespace declarations fails.
///
/// # Errors
///
/// This function can return a `LSPError` if:
/// * The document specified by `document_uri` cannot be found or loaded.
/// * The syntax tree for the document cannot be accessed.
/// * The query for extracting `PrefixDecl` fails to build or execute.
///
/// # Example
///
/// Given the following SPARQL query in the document located at `file://example.sparql`:
///
/// ```sparql
/// PREFIX ex: <http://example.org/>
/// PREFIX foaf: <http://xmlns.com/foaf/0.1/>
///
/// SELECT ?name WHERE {
///   ?person a foaf:Person .
///   ?person foaf:name ?name .
/// }
/// ```
///
/// Calling the function:
///
/// ```rust
/// let namespaces = get_declared_namespaces(&analysis_state, "file://example.sparql")?;
/// for (prefix, range) in namespaces {
///     println!("Found prefix: {} at range: {:?}", prefix, range);
/// }
/// ```
///
/// Would return:
///
/// ```text
/// Ok(vec![
///     ("ex".to_string(), Range { start: Position { line: 0, character: 7 }, end: Position { line: 0, character: 9 } }),
///     ("foaf".to_string(), Range { start: Position { line: 1, character: 7 }, end: Position { line: 1, character: 11 } }),
/// ])
/// ```
///
/// # Notes
///
/// The function assumes that the document is written in SPARQL syntax and uses
/// Tree-sitter for syntax tree traversal to locate namespace declarations.
pub(crate) fn get_declared_prefixes(
    server_state: &ServerState,
    document_uri: &str,
) -> Result<Vec<(String, Range)>, LSPError> {
    let (document, tree) = server_state.get_state(document_uri)?;
    let query = build_query("(PrefixDecl (PNAME_NS (PN_PREFIX) @prefix))")?;
    let mut query_cursor = QueryCursor::new();
    let mut captures = query_cursor.captures(&query, tree.root_node(), document.text.as_bytes());
    let mut namespaces: Vec<(String, Range)> = Vec::new();
    while let Some((mat, capture_index)) = captures.next() {
        let node = mat.captures[*capture_index].node;
        namespaces.push((
            node.utf8_text(document.text.as_bytes())
                .unwrap()
                .to_string(),
            Range::from_node(&node),
        ));
    }
    Ok(namespaces)
}

pub(crate) fn get_declared_uri_prefixes(
    server_state: &ServerState,
    document_uri: &str,
) -> Result<Vec<(String, Range)>, LSPError> {
    let (document, tree) = server_state.get_state(document_uri)?;
    let query = build_query("(PrefixDecl (PNAME_NS (PN_PREFIX)) (IRIREF) @uri)")?;
    let mut query_cursor = QueryCursor::new();
    let mut captures = query_cursor.captures(&query, tree.root_node(), document.text.as_bytes());
    let mut namespaces: Vec<(String, Range)> = Vec::new();
    while let Some((mat, capture_index)) = captures.next() {
        let node = mat.captures[*capture_index].node;
        namespaces.push((
            node.utf8_text(document.text.as_bytes())
                .unwrap()
                .to_string(),
            Range::from_node(&node),
        ));
    }
    Ok(namespaces)
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use tree_sitter::Parser;
    use tree_sitter_sparql::LANGUAGE;

    use crate::server::{
        anaysis::get_declared_prefixes, lsp::textdocument::TextDocumentItem, state::ServerState,
    };

    fn setup_state(text: &str) -> ServerState {
        let mut state = ServerState::new();
        let mut parser = Parser::new();
        if let Err(err) = parser.set_language(&LANGUAGE.into()) {
            log::error!("Could not initialize parser:\n{}", err)
        }
        let document = TextDocumentItem::new("uri", text);
        let tree = parser.parse(&document.text, None);
        state.add_document(document, tree);
        state
    }

    #[test]
    fn declared_namespaces() {
        let state = setup_state(indoc!(
            "PREFIX wdt: <iri>
                 PREFIX wd: <iri>
                 PREFIX wdt: <iri>

                 SELECT * {}"
        ));
        let declared_namesapces = get_declared_prefixes(&state, "uri").unwrap();
        assert_eq!(
            declared_namesapces
                .iter()
                .map(|(namespace, _range)| namespace)
                .collect::<Vec<&String>>(),
            vec!["wdt", "wd", "wdt"]
        );
    }
}
