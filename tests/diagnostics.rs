//! End-to-end tests for diagnostics
//!
//! Tests the textDocument/diagnostic LSP method.

mod harness;

use harness::TestClient;
use harness::runtime::run_lsp_test;
use indoc::indoc;
use serde_json::Value;

/// Extract all diagnostics with the given code from a diagnostic response.
fn diagnostics_with_code(response: &Value, code: &str) -> Vec<Value> {
    response["result"]["items"]
        .as_array()
        .expect("diagnostic response should have items array")
        .iter()
        .filter(|item| item["code"].as_str() == Some(code))
        .cloned()
        .collect()
}

// ========== groupby-star-selection ==========

const GROUPBY_STAR_CODE: &str = "groupby-star-selection";

#[test]
fn test_star_selection_with_group_by_is_flagged() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        let query = indoc!(
            "SELECT * WHERE {
               ?s ?p ?o
             }
             GROUP BY ?s"
        );
        client.open_document("file:///test.sparql", query).await;

        let id = client.diagnostics("file:///test.sparql").await;
        let response = client.get_response(id).expect("Should receive response");

        let diagnostics = diagnostics_with_code(&response, GROUPBY_STAR_CODE);
        assert_eq!(diagnostics.len(), 1);

        let diagnostic = &diagnostics[0];
        // NOTE: severity 1 = Error
        assert_eq!(diagnostic["severity"], 1);
        // NOTE: the range should cover the "*"
        assert_eq!(diagnostic["range"]["start"]["line"], 0);
        assert_eq!(diagnostic["range"]["start"]["character"], 7);
        assert_eq!(diagnostic["range"]["end"]["line"], 0);
        assert_eq!(diagnostic["range"]["end"]["character"], 8);
    });
}

#[test]
fn test_star_selection_without_group_by_is_not_flagged() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        let query = indoc!(
            "SELECT * WHERE {
               ?s ?p ?o
             }"
        );
        client.open_document("file:///test.sparql", query).await;

        let id = client.diagnostics("file:///test.sparql").await;
        let response = client.get_response(id).expect("Should receive response");

        assert!(diagnostics_with_code(&response, GROUPBY_STAR_CODE).is_empty());
    });
}

#[test]
fn test_variable_selection_with_group_by_is_not_flagged() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        let query = indoc!(
            "SELECT ?s WHERE {
               ?s ?p ?o
             }
             GROUP BY ?s"
        );
        client.open_document("file:///test.sparql", query).await;

        let id = client.diagnostics("file:///test.sparql").await;
        let response = client.get_response(id).expect("Should receive response");

        assert!(diagnostics_with_code(&response, GROUPBY_STAR_CODE).is_empty());
    });
}

#[test]
fn test_star_selection_in_grouped_exists_sub_select_is_flagged() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        let query = indoc!(
            "SELECT ?s WHERE {
               ?s ?p ?o
               FILTER EXISTS {
                 SELECT * WHERE {
                   ?a ?b ?c
                 }
                 GROUP BY ?a
               }
             }"
        );
        client.open_document("file:///test.sparql", query).await;

        let id = client.diagnostics("file:///test.sparql").await;
        let response = client.get_response(id).expect("Should receive response");

        assert_eq!(diagnostics_with_code(&response, GROUPBY_STAR_CODE).len(), 1);
    });
}

#[test]
fn test_star_selection_in_grouped_sub_select_is_flagged() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        let query = indoc!(
            "SELECT ?s WHERE {
               {
                 SELECT * WHERE {
                   ?s ?p ?o
                 }
                 GROUP BY ?s
               }
             }"
        );
        client.open_document("file:///test.sparql", query).await;

        let id = client.diagnostics("file:///test.sparql").await;
        let response = client.get_response(id).expect("Should receive response");

        let diagnostics = diagnostics_with_code(&response, GROUPBY_STAR_CODE);
        assert_eq!(diagnostics.len(), 1);

        // NOTE: the range should cover the "*" of the sub-select
        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic["range"]["start"]["line"], 2);
        assert_eq!(diagnostic["range"]["start"]["character"], 11);
        assert_eq!(diagnostic["range"]["end"]["line"], 2);
        assert_eq!(diagnostic["range"]["end"]["character"], 12);
    });
}
