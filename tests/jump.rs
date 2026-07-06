//! End-to-end tests for jump navigation
//!
//! Tests the qlueLs/jump LSP extension. The server formats the document,
//! computes the jump target on the formatted document and returns the
//! edits together with the final cursor position.

mod harness;

use harness::TestClient;
use harness::runtime::run_lsp_test;
use indoc::indoc;

#[test]
fn test_jump_next_formats_and_jumps_past_query() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // NOTE: cursor on the whitespace-only line the formatter removes
        let query = "SELECT * WHERE {\n  ?s ?p ?o\n  \n}";
        client.open_document("file:///test.sparql", query).await;

        let id = client.jump("file:///test.sparql", 2, 2, false).await;
        let response = client.get_response(id).expect("Should receive response");
        let result = &response["result"];

        let edits = result["edits"].as_array().expect("edits should be array");
        assert_eq!(edits.len(), 1);
        assert_eq!(
            edits[0]["newText"].as_str().unwrap(),
            "SELECT * WHERE {\n  ?s ?p ?o\n}\n\n"
        );
        // NOTE: cursor on the new line below the query
        assert_eq!(result["position"]["line"], 3);
        assert_eq!(result["position"]["character"], 0);
    });
}

#[test]
fn test_jump_prev_from_body_to_select_clause() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        let query = indoc!(
            "SELECT * WHERE {
               ?s ?p ?o
            }"
        );
        client.open_document("file:///test.sparql", query).await;

        let id = client.jump("file:///test.sparql", 1, 2, true).await;
        let response = client.get_response(id).expect("Should receive response");
        let result = &response["result"];

        let edits = result["edits"].as_array().expect("edits should be array");
        assert_eq!(edits.len(), 1);
        assert_eq!(
            edits[0]["newText"].as_str().unwrap(),
            "SELECT *  WHERE {\n  ?s ?p ?o\n}\n"
        );
        // NOTE: cursor before the WHERE clause, a " " is inserted after it
        assert_eq!(result["position"]["line"], 0);
        assert_eq!(result["position"]["character"], 9);
    });
}

#[test]
fn test_jump_next_into_group_graph_pattern() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        let query = indoc!(
            "SELECT * WHERE {
               ?s ?p ?o
            }"
        );
        client.open_document("file:///test.sparql", query).await;

        // NOTE: cursor at the end of the select clause
        let id = client.jump("file:///test.sparql", 0, 8, false).await;
        let response = client.get_response(id).expect("Should receive response");
        let result = &response["result"];

        let edits = result["edits"].as_array().expect("edits should be array");
        assert_eq!(edits.len(), 1);
        assert_eq!(
            edits[0]["newText"].as_str().unwrap(),
            "SELECT * WHERE {\n  ?s ?p ?o\n  \n}\n"
        );
        // NOTE: cursor on the new line inside the group graph pattern
        assert_eq!(result["position"]["line"], 2);
        assert_eq!(result["position"]["character"], 2);
    });
}

#[test]
fn test_jump_applies_format_before_computing_target() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // NOTE: unformatted query, cursor at the very end
        let query = "SELECT*WHERE{?a ?b ?c}";
        client.open_document("file:///test.sparql", query).await;

        let id = client.jump("file:///test.sparql", 0, 22, false).await;
        let response = client.get_response(id).expect("Should receive response");
        let result = &response["result"];

        let edits = result["edits"].as_array().expect("edits should be array");
        assert_eq!(edits.len(), 1);
        // NOTE: the jump wraps around to the end of the select clause,
        // which inserts a " " before the cursor
        assert_eq!(
            edits[0]["newText"].as_str().unwrap(),
            "SELECT *  WHERE {\n  ?a ?b ?c\n}\n"
        );
        assert_eq!(result["position"]["line"], 0);
        assert_eq!(result["position"]["character"], 9);
    });
}

#[test]
fn test_jump_on_empty_document_does_not_crash() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client.open_document("file:///test.sparql", "").await;

        let id = client.jump("file:///test.sparql", 0, 0, false).await;
        let response = client.get_response(id).expect("Should receive response");
        let result = &response["result"];

        assert!(result["edits"].is_array());
    });
}
