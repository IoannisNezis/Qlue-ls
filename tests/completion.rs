//! End-to-end tests for completion filtering
//!
//! Tests that keyword completions are filtered based on search term prefix.

mod harness;

use harness::runtime::run_lsp_test;
use harness::TestClient;
use serde_json::Value;

/// Helper to extract completion labels from a completion response
fn get_completion_labels(response: &Value) -> Vec<String> {
    response["result"]["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item["label"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Helper to check if a completion label exists in the response
fn has_completion_label(response: &Value, label: &str) -> bool {
    get_completion_labels(response).contains(&label.to_string())
}

#[test]
fn test_completion_filter_prefix_returns_filter() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Open a document with "FI" typed in subject position
        // The cursor is at the end of "FI"
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { FI }")
            .await;

        // Request completion at position after "FI" (line 0, character 17)
        let id = client.complete("file:///test.sparql", 0, 17).await;
        let response = client.get_response(id).expect("Should receive completion response");

        assert!(
            response.get("result").is_some(),
            "Completion should return a result: {:?}",
            response
        );

        // "FI" should match FILTER
        assert!(
            has_completion_label(&response, "FILTER"),
            "Should suggest FILTER for prefix 'FI', got: {:?}",
            get_completion_labels(&response)
        );

        // "FI" should NOT match other keywords like BIND, OPTIONAL, etc.
        assert!(
            !has_completion_label(&response, "BIND"),
            "Should NOT suggest BIND for prefix 'FI'"
        );
        assert!(
            !has_completion_label(&response, "OPTIONAL"),
            "Should NOT suggest OPTIONAL for prefix 'FI'"
        );
        assert!(
            !has_completion_label(&response, "VALUES"),
            "Should NOT suggest VALUES for prefix 'FI'"
        );
    });
}

#[test]
fn test_completion_non_keyword_prefix_excludes_keywords() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Open a document with "Germany" typed in subject position
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { Germany }")
            .await;

        // Request completion at position after "Germany" (line 0, character 24)
        let id = client.complete("file:///test.sparql", 0, 24).await;
        let response = client.get_response(id).expect("Should receive completion response");

        assert!(
            response.get("result").is_some(),
            "Completion should return a result: {:?}",
            response
        );

        // "Germany" should NOT match any keywords
        assert!(
            !has_completion_label(&response, "FILTER"),
            "Should NOT suggest FILTER for 'Germany'"
        );
        assert!(
            !has_completion_label(&response, "BIND"),
            "Should NOT suggest BIND for 'Germany'"
        );
        assert!(
            !has_completion_label(&response, "OPTIONAL"),
            "Should NOT suggest OPTIONAL for 'Germany'"
        );
        assert!(
            !has_completion_label(&response, "VALUES"),
            "Should NOT suggest VALUES for 'Germany'"
        );
        assert!(
            !has_completion_label(&response, "SERVICE"),
            "Should NOT suggest SERVICE for 'Germany'"
        );
        assert!(
            !has_completion_label(&response, "MINUS"),
            "Should NOT suggest MINUS for 'Germany'"
        );
        assert!(
            !has_completion_label(&response, "UNION"),
            "Should NOT suggest UNION for 'Germany'"
        );
    });
}

#[test]
fn test_completion_optional_prefix_returns_optional() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Open a document with "OP" typed in subject position
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { OP }")
            .await;

        // Request completion at position after "OP" (line 0, character 18)
        let id = client.complete("file:///test.sparql", 0, 18).await;
        let response = client.get_response(id).expect("Should receive completion response");

        assert!(
            response.get("result").is_some(),
            "Completion should return a result: {:?}",
            response
        );

        // "OP" should match OPTIONAL
        assert!(
            has_completion_label(&response, "OPTIONAL"),
            "Should suggest OPTIONAL for prefix 'OP', got: {:?}",
            get_completion_labels(&response)
        );

        // "OP" should NOT match other keywords
        assert!(
            !has_completion_label(&response, "FILTER"),
            "Should NOT suggest FILTER for prefix 'OP'"
        );
        assert!(
            !has_completion_label(&response, "BIND"),
            "Should NOT suggest BIND for prefix 'OP'"
        );
    });
}

#[test]
fn test_completion_case_insensitive_prefix() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Open a document with lowercase "fi" typed in subject position
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { fi }")
            .await;

        // Request completion at position after "fi" (line 0, character 18)
        let id = client.complete("file:///test.sparql", 0, 18).await;
        let response = client.get_response(id).expect("Should receive completion response");

        assert!(
            response.get("result").is_some(),
            "Completion should return a result: {:?}",
            response
        );

        // lowercase "fi" should match FILTER (case insensitive)
        assert!(
            has_completion_label(&response, "FILTER"),
            "Should suggest FILTER for lowercase prefix 'fi', got: {:?}",
            get_completion_labels(&response)
        );
    });
}

#[test]
fn test_completion_bind_prefix_returns_bind() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Open a document with "BI" typed in subject position
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { BI }")
            .await;

        // Request completion at position after "BI" (line 0, character 18)
        let id = client.complete("file:///test.sparql", 0, 18).await;
        let response = client.get_response(id).expect("Should receive completion response");

        assert!(
            response.get("result").is_some(),
            "Completion should return a result: {:?}",
            response
        );

        // "BI" should match BIND
        assert!(
            has_completion_label(&response, "BIND"),
            "Should suggest BIND for prefix 'BI', got: {:?}",
            get_completion_labels(&response)
        );

        // "BI" should NOT match other keywords
        assert!(
            !has_completion_label(&response, "FILTER"),
            "Should NOT suggest FILTER for prefix 'BI'"
        );
        assert!(
            !has_completion_label(&response, "OPTIONAL"),
            "Should NOT suggest OPTIONAL for prefix 'BI'"
        );
    });
}

#[test]
fn test_completion_s_prefix_returns_service_and_sub_select() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Open a document with "S" typed in subject position
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { S }")
            .await;

        // Request completion at position after "S" (line 0, character 17)
        let id = client.complete("file:///test.sparql", 0, 17).await;
        let response = client.get_response(id).expect("Should receive completion response");

        assert!(
            response.get("result").is_some(),
            "Completion should return a result: {:?}",
            response
        );

        // "S" should match SERVICE and Sub select
        assert!(
            has_completion_label(&response, "SERVICE"),
            "Should suggest SERVICE for prefix 'S', got: {:?}",
            get_completion_labels(&response)
        );
        assert!(
            has_completion_label(&response, "Sub select"),
            "Should suggest 'Sub select' for prefix 'S', got: {:?}",
            get_completion_labels(&response)
        );

        // "S" should NOT match other keywords like FILTER, BIND
        assert!(
            !has_completion_label(&response, "FILTER"),
            "Should NOT suggest FILTER for prefix 'S'"
        );
        assert!(
            !has_completion_label(&response, "BIND"),
            "Should NOT suggest BIND for prefix 'S'"
        );
    });
}

#[test]
fn test_completion_solution_modifier_group_prefix() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Open a document with "GR" after the WHERE clause (solution modifier position)
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { ?s ?p ?o } GR")
            .await;

        // Request completion at position after "GR" (line 0, character 30)
        let id = client.complete("file:///test.sparql", 0, 30).await;
        let response = client.get_response(id).expect("Should receive completion response");

        assert!(
            response.get("result").is_some(),
            "Completion should return a result: {:?}",
            response
        );

        // "GR" should match GROUP BY
        assert!(
            has_completion_label(&response, "GROUP BY"),
            "Should suggest 'GROUP BY' for prefix 'GR', got: {:?}",
            get_completion_labels(&response)
        );

        // "GR" should NOT match other solution modifiers
        assert!(
            !has_completion_label(&response, "ORDER BY"),
            "Should NOT suggest 'ORDER BY' for prefix 'GR'"
        );
        assert!(
            !has_completion_label(&response, "LIMIT"),
            "Should NOT suggest LIMIT for prefix 'GR'"
        );
    });
}

#[test]
fn test_completion_solution_modifier_non_keyword_excludes_all() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Open a document with "xyz" after the WHERE clause (solution modifier position)
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { ?s ?p ?o } xyz")
            .await;

        // Request completion at position after "xyz" (line 0, character 31)
        let id = client.complete("file:///test.sparql", 0, 31).await;
        let response = client.get_response(id).expect("Should receive completion response");

        assert!(
            response.get("result").is_some(),
            "Completion should return a result: {:?}",
            response
        );

        // "xyz" should NOT match any solution modifier keywords
        assert!(
            !has_completion_label(&response, "GROUP BY"),
            "Should NOT suggest 'GROUP BY' for 'xyz'"
        );
        assert!(
            !has_completion_label(&response, "ORDER BY"),
            "Should NOT suggest 'ORDER BY' for 'xyz'"
        );
        assert!(
            !has_completion_label(&response, "HAVING"),
            "Should NOT suggest HAVING for 'xyz'"
        );
        assert!(
            !has_completion_label(&response, "LIMIT"),
            "Should NOT suggest LIMIT for 'xyz'"
        );
        assert!(
            !has_completion_label(&response, "OFFSET"),
            "Should NOT suggest OFFSET for 'xyz'"
        );
    });
}
