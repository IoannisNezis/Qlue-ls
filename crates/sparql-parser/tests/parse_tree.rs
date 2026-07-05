//! Snapshot tests for the concrete syntax trees produced by the parser.
//!
//! Each test parses an input and snapshots the resulting tree (plus any
//! parse errors) with `insta`. On first run, or after a grammar change,
//! review the snapshots with:
//!
//! ```bash
//! cargo insta review
//! ```
//!
//! Snapshots live in `tests/snapshots/`.

use ll_sparql_parser::{parse, parse_query, parse_update};

/// Render the syntax tree and parse errors into a single snapshot string.
///
/// Also asserts losslessness: the tree must reproduce the input verbatim.
fn render(
    input: &str,
    tree: ll_sparql_parser::SyntaxNode,
    errors: Vec<impl std::fmt::Debug>,
) -> String {
    assert_eq!(tree.text().to_string(), input, "parse tree is not lossless");
    let mut out = format!("{tree:#?}");
    if !errors.is_empty() {
        out.push_str("---\n");
        for error in &errors {
            out.push_str(&format!("{error:?}\n"));
        }
    }
    out
}

/// Snapshot-test an input parsed as a query (`QueryUnit`).
fn check_query(input: &str) -> String {
    let (tree, errors) = parse_query(input);
    render(input, tree, errors)
}

/// Snapshot-test an input parsed as an update (`UpdateUnit`).
fn check_update(input: &str) -> String {
    let (tree, errors) = parse_update(input);
    render(input, tree, errors)
}

/// Snapshot-test an input with auto-detected operation type.
fn check_auto(input: &str) -> String {
    let (tree, errors) = parse(input);
    render(input, tree, errors)
}

#[test]
fn select_simple() {
    insta::assert_snapshot!(check_query("SELECT * WHERE { ?s ?p ?o }"));
}

#[test]
fn select_with_prefix() {
    insta::assert_snapshot!(check_query(
        "PREFIX foaf: <http://xmlns.com/foaf/0.1/>\nSELECT ?name WHERE { ?person foaf:name ?name }"
    ));
}

#[test]
fn insert_data() {
    insta::assert_snapshot!(check_update(r#"INSERT DATA { <a> <b> "c" . }"#));
}

#[test]
fn auto_detect_ask() {
    insta::assert_snapshot!(check_auto("ASK { ?s ?p ?o }"));
}

#[test]
fn error_recovery_incomplete_select() {
    insta::assert_snapshot!(check_query("SELECT ?s WHERE { ?s"));
}

#[test]
fn error_recovery_garbage() {
    insta::assert_snapshot!(check_query("SELECT } WHERE ?? {"));
}

#[test]
fn error_recovery_incomplete_select_clause() {
    insta::assert_snapshot!(check_query("SELECT ?a ? WHERE { }"));
}
