//! Add rdfs:label code action
//! Adds rdfs:label triple and LANG filter for a variable
//!
//! **Example:**
//!
//! ?thing wdt:P17 ?country .
//!
//! ----------------
//!
//! ?thing wdt:P17 ?country .
//! ?country rdfs:label ?country_label .
//! FILTER (LANG(?country_label) = "en")

use crate::server::{
    analysis::namespace_is_declared,
    lsp::{
        CodeAction, WorkspaceEdit,
        textdocument::{Position, Range, TextDocumentItem, TextEdit},
    },
    state::ServerState,
};
use ll_sparql_parser::{
    ast::{AstNode, Var},
    syntax_kind::SyntaxKind,
};
use std::collections::HashMap;

const RDFS_PREFIX: &str = "rdfs";
const RDFS_NAMESPACE: &str = "http://www.w3.org/2000/01/rdf-schema#";

pub(super) fn code_action(
    var: &Var,
    server_state: &ServerState,
    document: &TextDocumentItem,
) -> Option<CodeAction> {
    let triple = var.triple()?;
    // Extract indentation from the subject of the triple
    let subject = triple.subject()?;
    let subject_start: usize = subject.syntax().text_range().start().into();
    let line_start = document.text[..subject_start]
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0);
    let indentation = &document.text[line_start..subject_start];

    let var_name = var.var_name();
    let label_var = format!("?{var_name}_label");
    let lang_filter = format!(r#"FILTER (LANG({label_var}) = "en")"#);
    let label_triple = format!(
        " .\n{indentation}{} rdfs:label {label_var} . {lang_filter}",
        var.syntax()
    );

    let start = Position::from_byte_index(
        triple.properties_list_path().and_then(|path| {
            path.properties()
                .last()
                .map(|last| last.object.syntax().text_range().end())
        })?,
        &document.text,
    )?;

    let end = triple
        .syntax()
        .next_sibling_or_token()
        .and_then(|next| {
            if next.kind().is_trivia() {
                next.next_sibling_or_token()
            } else {
                Some(next)
            }
        })
        .filter(|next| matches!(next.kind(), SyntaxKind::Dot))
        .and_then(|next| Position::from_byte_index(next.text_range().end(), &document.text))
        .unwrap_or(start.clone());
    let mut edits = vec![TextEdit::new(Range { start, end }, &label_triple)];

    // Add rdfs prefix if not declared
    if !namespace_is_declared(server_state, &document.uri, RDFS_PREFIX).unwrap_or(true) {
        edits.push(TextEdit::new(
            Range::new(0, 0, 0, 0),
            &format!("PREFIX {RDFS_PREFIX}: <{RDFS_NAMESPACE}>\n"),
        ));
    }

    Some(CodeAction {
        title: "Add rdfs:label".to_string(),
        kind: None,
        diagnostics: vec![],
        edit: WorkspaceEdit {
            changes: Some(HashMap::from_iter([(document.uri.to_string(), edits)])),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::lsp::textdocument::TextDocumentItem;
    use indoc::indoc;
    use ll_sparql_parser::{ast::AstNode, parse_query};

    fn setup_state(text: &str) -> ServerState {
        let mut state = ServerState::new();
        let document = TextDocumentItem::new("uri", text);
        state.add_document(document);
        state
    }

    fn find_var_by_name(text: &str, var_name: &str) -> Option<Var> {
        let root = parse_query(text);
        root.descendants()
            .filter_map(Var::cast)
            .find(|v| v.syntax().text().to_string() == var_name)
    }

    #[test]
    fn test_add_label_without_rdfs_prefix() {
        let text = indoc!(
            "SELECT * WHERE {
               ?thing wdt:P17 ?country .
             }"
        );
        let state = setup_state(text);
        let document = state.get_document("uri").unwrap().clone();

        let var = find_var_by_name(text, "?country").unwrap();

        let action = code_action(&var, &state, &document).unwrap();

        assert_eq!(action.title, "Add rdfs:label");

        let edits = action.edit.changes.unwrap();
        let uri_edits = edits.get("uri").unwrap();

        // Should have 2 edits: label triple + filter, and prefix declaration
        assert_eq!(uri_edits.len(), 2);

        // Check that the label triple and filter are on the same line with proper indentation
        let insert_edit = &uri_edits[0];
        assert!(
            insert_edit
                .new_text
                .contains("?country rdfs:label ?country_label . FILTER")
        );
        // Verify indentation matches the original subject (2 spaces from indoc normalization)
        assert!(insert_edit.new_text.starts_with(" .\n  ?country"));

        // Check that prefix is added
        let prefix_edit = &uri_edits[1];
        assert_eq!(
            prefix_edit.new_text,
            "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n"
        );
        assert_eq!(prefix_edit.range, Range::new(0, 0, 0, 0));
    }

    #[test]
    fn test_add_label_with_rdfs_prefix() {
        let text = indoc!(
            "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
             SELECT * WHERE {
               ?thing wdt:P17 ?country .
             }"
        );
        let state = setup_state(text);
        let document = state.get_document("uri").unwrap().clone();

        let var = find_var_by_name(text, "?country").unwrap();

        let action = code_action(&var, &state, &document).unwrap();

        let edits = action.edit.changes.unwrap();
        let uri_edits = edits.get("uri").unwrap();

        // Should only have 1 edit: label triple + filter (no prefix needed)
        assert_eq!(uri_edits.len(), 1);
    }

    #[test]
    fn test_label_variable_naming() {
        let text = "SELECT * WHERE { ?person a ?type . }";
        let state = setup_state(text);
        let document = state.get_document("uri").unwrap().clone();

        let var = find_var_by_name(text, "?person").unwrap();

        let action = code_action(&var, &state, &document).unwrap();

        let edits = action.edit.changes.unwrap();
        let uri_edits = edits.get("uri").unwrap();
        let insert_edit = &uri_edits[0];

        // Check that triple and filter are on same line
        assert!(
            insert_edit.new_text.contains(
                r#"?person rdfs:label ?person_label . FILTER (LANG(?person_label) = "en")"#
            )
        );
    }
}
