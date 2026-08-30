//! End-to-end tests for completion filtering
//!
//! Tests that keyword completions are filtered based on search term prefix.

mod harness;

use harness::TestClient;
use harness::runtime::run_lsp_test;
use serde_json::{Value, json};

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
            //                                     012345678901234567890
            .open_document("file:///test.sparql", "SELECT * WHERE { FI }")
            .await;

        // Request completion at position after "FI" (line 0, character 19)
        let id = client.complete("file:///test.sparql", 0, 19).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

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
            //                                     0000000000111111111122222222223
            //                                     0123456789012345678901234567890
            .open_document("file:///test.sparql", "SELECT * WHERE { Germany }")
            .await;

        // Request completion at position after "Germany" (line 0, character 24)
        let id = client.complete("file:///test.sparql", 0, 24).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

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
            //                                     0000000000111111111122222222223
            //                                     0123456789012345678901234567890
            .open_document("file:///test.sparql", "SELECT * WHERE { OP }")
            .await;

        // Request completion at position after "OP" (line 0, character 19)
        let id = client.complete("file:///test.sparql", 0, 19).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

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
            //                                     0000000000111111111122222222223
            //                                     0123456789012345678901234567890
            .open_document("file:///test.sparql", "SELECT * WHERE { fi }")
            .await;

        // Request completion at position after "fi" (line 0, character 19)
        let id = client.complete("file:///test.sparql", 0, 19).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

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
            //                                     0000000000111111111122222222223
            //                                     0123456789012345678901234567890
            .open_document("file:///test.sparql", "SELECT * WHERE { BI }")
            .await;

        // Request completion at position after "BI" (line 0, character 19)
        let id = client.complete("file:///test.sparql", 0, 19).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

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
            //                                     0000000000111111111122222222223
            //                                     0123456789012345678901234567890
            .open_document("file:///test.sparql", "SELECT * WHERE { S }")
            .await;

        // Request completion at position after "S" (line 0, character 18)
        let id = client.complete("file:///test.sparql", 0, 18).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

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

/// Helper: labels that contain the given substring
fn labels_containing(response: &Value, substr: &str) -> Vec<String> {
    get_completion_labels(response)
        .into_iter()
        .filter(|label| label.contains(substr))
        .collect()
}

#[test]
fn test_aggregate_completion_with_group_by_no_prefix() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Cursor in the (empty) select binding position, with GROUP BY present
        client
            //                                     0000000000111111111122222222223333333333
            //                                     0123456789012345678901234567890123456789
            .open_document(
                "file:///test.sparql",
                "SELECT  WHERE { ?s ?p ?o } GROUP BY ?s",
            )
            .await;

        let id = client.complete("file:///test.sparql", 0, 7).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        // All aggregate functions should be offered for the non-grouped variables
        for aggregate in ["COUNT", "SUM", "MIN", "MAX", "AVG", "SAMPLE"] {
            assert!(
                !labels_containing(&response, &format!("{aggregate}(")).is_empty(),
                "Should suggest {aggregate} aggregate, got: {:?}",
                get_completion_labels(&response)
            );
        }
    });
}

#[test]
fn test_aggregate_completion_partial_s_suggests_sum_and_sample() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Partially written aggregate "(S" in the select clause
        client
            //                                     0000000000111111111122222222223333333333444
            //                                     0123456789012345678901234567890123456789012
            .open_document(
                "file:///test.sparql",
                "SELECT (S WHERE { ?s ?p ?o } GROUP BY ?s",
            )
            .await;

        // Cursor right after "(S" (line 0, character 9)
        let id = client.complete("file:///test.sparql", 0, 9).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        // "S" should match SUM and SAMPLE
        assert!(
            !labels_containing(&response, "SUM(").is_empty(),
            "Should suggest SUM for partial '(S', got: {:?}",
            get_completion_labels(&response)
        );
        assert!(
            !labels_containing(&response, "SAMPLE(").is_empty(),
            "Should suggest SAMPLE for partial '(S', got: {:?}",
            get_completion_labels(&response)
        );

        // "S" should NOT match the other aggregates
        for aggregate in ["COUNT", "MIN", "MAX", "AVG"] {
            assert!(
                labels_containing(&response, &format!("{aggregate}(")).is_empty(),
                "Should NOT suggest {aggregate} for partial '(S', got: {:?}",
                get_completion_labels(&response)
            );
        }
    });
}

#[test]
fn test_aggregate_completion_partial_su_suggests_only_sum() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            //                                     0000000000111111111122222222223333333333444
            //                                     0123456789012345678901234567890123456789012
            .open_document(
                "file:///test.sparql",
                "SELECT (SU WHERE { ?s ?p ?o } GROUP BY ?s",
            )
            .await;

        // Cursor right after "(SU" (line 0, character 10)
        let id = client.complete("file:///test.sparql", 0, 10).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        assert!(
            !labels_containing(&response, "SUM(").is_empty(),
            "Should suggest SUM for partial '(SU', got: {:?}",
            get_completion_labels(&response)
        );
        for aggregate in ["COUNT", "MIN", "MAX", "AVG", "SAMPLE"] {
            assert!(
                labels_containing(&response, &format!("{aggregate}(")).is_empty(),
                "Should NOT suggest {aggregate} for partial '(SU', got: {:?}",
                get_completion_labels(&response)
            );
        }
    });
}

#[test]
fn test_aggregate_completion_partial_lowercase() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            //                                     0000000000111111111122222222223333333333444
            //                                     0123456789012345678901234567890123456789012
            .open_document(
                "file:///test.sparql",
                "SELECT (sa WHERE { ?s ?p ?o } GROUP BY ?s",
            )
            .await;

        // Cursor right after "(sa" (line 0, character 10)
        let id = client.complete("file:///test.sparql", 0, 10).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        assert!(
            !labels_containing(&response, "SAMPLE(").is_empty(),
            "Should suggest SAMPLE for partial '(sa' (case-insensitive), got: {:?}",
            get_completion_labels(&response)
        );
        assert!(
            labels_containing(&response, "SUM(").is_empty(),
            "Should NOT suggest SUM for partial '(sa', got: {:?}",
            get_completion_labels(&response)
        );
    });
}

#[test]
fn test_aggregate_completion_partial_c_includes_count_star() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            //                                     0000000000111111111122222222223333333333444
            //                                     0123456789012345678901234567890123456789012
            .open_document(
                "file:///test.sparql",
                "SELECT (C WHERE { ?s ?p ?o } GROUP BY ?s",
            )
            .await;

        // Cursor right after "(C" (line 0, character 9)
        let id = client.complete("file:///test.sparql", 0, 9).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        assert!(
            !labels_containing(&response, "COUNT(").is_empty(),
            "Should suggest COUNT for partial '(C', got: {:?}",
            get_completion_labels(&response)
        );
        assert!(
            has_completion_label(&response, "(COUNT(*) AS ?count)"),
            "Should suggest '(COUNT(*) AS ?count)' for partial '(C', got: {:?}",
            get_completion_labels(&response)
        );
        for aggregate in ["SUM", "MIN", "MAX", "AVG", "SAMPLE"] {
            assert!(
                labels_containing(&response, &format!("{aggregate}(")).is_empty(),
                "Should NOT suggest {aggregate} for partial '(C', got: {:?}",
                get_completion_labels(&response)
            );
        }
    });
}

#[test]
fn test_aggregate_completion_partial_without_paren() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Partial aggregate without the opening paren
        client
            //                                     0000000000111111111122222222223333333333
            //                                     0123456789012345678901234567890123456789
            .open_document(
                "file:///test.sparql",
                "SELECT S WHERE { ?s ?p ?o } GROUP BY ?s",
            )
            .await;

        // Cursor right after "S" (line 0, character 8)
        let id = client.complete("file:///test.sparql", 0, 8).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        assert!(
            !labels_containing(&response, "SUM(").is_empty(),
            "Should suggest SUM for partial 'S', got: {:?}",
            get_completion_labels(&response)
        );
        assert!(
            !labels_containing(&response, "SAMPLE(").is_empty(),
            "Should suggest SAMPLE for partial 'S', got: {:?}",
            get_completion_labels(&response)
        );
        for aggregate in ["COUNT", "MIN", "MAX", "AVG"] {
            assert!(
                labels_containing(&response, &format!("{aggregate}(")).is_empty(),
                "Should NOT suggest {aggregate} for partial 'S', got: {:?}",
                get_completion_labels(&response)
            );
        }
    });
}

#[test]
fn test_no_binding_snippets_inside_partial_aggregate() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Cursor inside a partially written aggregate call: this is an
        // expression position, not a fresh select binding. Offering the
        // "(AGG(?var) AS ?alias)" snippets here would replace the inner
        // paren and produce garbage like "(SUM(SUM(?o) AS ?sum_o)".
        client
            //                                     0000000000111111111122222222223333333333444
            //                                     0123456789012345678901234567890123456789012
            .open_document(
                "file:///test.sparql",
                "SELECT (SUM( WHERE { ?s ?p ?o } GROUP BY ?s",
            )
            .await;

        // Cursor right after the inner "(" (line 0, character 12)
        let id = client.complete("file:///test.sparql", 0, 12).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        assert!(
            labels_containing(&response, " AS ?").is_empty(),
            "Should NOT suggest binding snippets inside an aggregate call, got: {:?}",
            get_completion_labels(&response)
        );
        assert!(
            !has_completion_label(&response, "REDUCED"),
            "Should NOT suggest REDUCED inside an aggregate call"
        );
    });
}

#[test]
fn test_no_aggregate_completion_without_group_by_and_selected_var() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // A variable is already selected and there is no GROUP BY:
        // aggregates would be invalid here
        client
            //                                     000000000011111111112222222222
            //                                     012345678901234567890123456789
            .open_document("file:///test.sparql", "SELECT ?s  WHERE { ?s ?p ?o }")
            .await;

        let id = client.complete("file:///test.sparql", 0, 10).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        for aggregate in ["COUNT", "SUM", "MIN", "MAX", "AVG", "SAMPLE"] {
            assert!(
                labels_containing(&response, &format!("{aggregate}(")).is_empty(),
                "Should NOT suggest {aggregate} without GROUP BY when a variable is selected, got: {:?}",
                get_completion_labels(&response)
            );
        }
    });
}

#[test]
fn test_completion_solution_modifier_group_prefix() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Open a document with "GR" after the WHERE clause (solution modifier position)
        client
            //                                     0000000000111111111122222222223
            //                                     0123456789012345678901234567890
            .open_document("file:///test.sparql", "SELECT * WHERE { ?s ?p ?o } GR")
            .await;

        // Request completion at position after "GR" (line 0, character 30)
        let id = client.complete("file:///test.sparql", 0, 30).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

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
            //                                     00000000001111111111222222222233
            //                                     01234567890123456789012345678901
            .open_document("file:///test.sparql", "SELECT * WHERE { ?s ?p ?o } xyz")
            .await;

        // Request completion at position after "xyz" (line 0, character 31)
        let id = client.complete("file:///test.sparql", 0, 31).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

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

// --- GroupCondition (GROUP BY) completion tests ---

#[test]
fn test_group_condition_suggests_visible_variables() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            //                                     0000000000111111111122222222223333333333
            //                                     0123456789012345678901234567890123456789
            .open_document(
                "file:///test.sparql",
                "SELECT * WHERE { ?s ?p ?o } GROUP BY ",
            )
            .await;

        // Cursor right after "GROUP BY " (line 0, character 37)
        let id = client.complete("file:///test.sparql", 0, 37).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        for variable in ["?s", "?p", "?o"] {
            assert!(
                !labels_containing(&response, variable).is_empty(),
                "Should suggest {variable} after GROUP BY, got: {:?}",
                get_completion_labels(&response)
            );
        }
    });
}

#[test]
fn test_group_condition_second_condition_suggests_variables() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            //   0000000000111111111122222222223333333333444
            //   0123456789012345678901234567890123456789012
            .open_document(
                "file:///test.sparql",
                "SELECT * WHERE { ?s ?p ?o } GROUP BY ?s ?",
            )
            .await;

        // Cursor after "GROUP BY ?s " (line 0, character 41)
        let id = client.complete("file:///test.sparql", 0, 41).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        for variable in ["?p", "?o"] {
            assert!(
                !labels_containing(&response, variable).is_empty(),
                "Should suggest {variable} as second group condition, got: {:?}",
                get_completion_labels(&response)
            );
        }
    });
}

#[test]
fn test_group_condition_partial_variable_filters() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            //                                     000000000011111111112222222222333333333344444444
            //                                     012345678901234567890123456789012345678901234567
            .open_document(
                "file:///test.sparql",
                "SELECT * WHERE { ?name ?p ?o } GROUP BY ?n",
            )
            .await;

        // Cursor right after "?n" (line 0, character 42)
        let id = client.complete("file:///test.sparql", 0, 42).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        assert!(
            !labels_containing(&response, "?name").is_empty(),
            "Should suggest ?name for partial '?n', got: {:?}",
            get_completion_labels(&response)
        );
        assert!(
            labels_containing(&response, "?p").is_empty(),
            "Should NOT suggest ?p for partial '?n', got: {:?}",
            get_completion_labels(&response)
        );
    });
}

#[test]
fn test_group_condition_nested_sub_select_suggests_inner_variables() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            //                                     0000000000111111111122222222223333333333444444444455555555556666666666777777777788888888889999999999
            //                                     0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789
            .open_document(
                "file:///test.sparql",
                "SELECT * WHERE { ?a ?b ?c { SELECT * WHERE { ?x ?y ?z { SELECT * WHERE { ?s ?p ?o } GROUP BY ",
            )
            .await;

        // Cursor right after the innermost "GROUP BY " (line 0, character 93)
        let id = client.complete("file:///test.sparql", 0, 93).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        // Variables of the innermost sub-select should be suggested
        for variable in ["?s", "?p", "?o"] {
            assert!(
                !labels_containing(&response, variable).is_empty(),
                "Should suggest {variable} in nested sub-select GROUP BY, got: {:?}",
                get_completion_labels(&response)
            );
        }
        // Variables from enclosing scopes are not visible inside the sub-select
        for variable in ["?a", "?x"] {
            assert!(
                labels_containing(&response, variable).is_empty(),
                "Should NOT suggest outer variable {variable} in nested sub-select GROUP BY, got: {:?}",
                get_completion_labels(&response)
            );
        }
    });
}

#[test]
fn test_group_condition_no_keyword_completions() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            //                                     0000000000111111111122222222223333333333
            //                                     0123456789012345678901234567890123456789
            .open_document(
                "file:///test.sparql",
                "SELECT * WHERE { ?s ?p ?o } GROUP BY ",
            )
            .await;

        let id = client.complete("file:///test.sparql", 0, 37).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        // Solution modifier keywords are not valid group conditions
        for keyword in ["GROUP BY", "ORDER BY", "LIMIT", "OFFSET", "FILTER"] {
            assert!(
                !has_completion_label(&response, keyword),
                "Should NOT suggest '{keyword}' as a group condition, got: {:?}",
                get_completion_labels(&response)
            );
        }
    });
}

// NOTE: object variable completion replacements
//
// In object position the predicate's local name is offered as a variable name,
// after the configured `replacements.objectVariable` regexes and the built in
// snake_case conversion have been applied.
// See `Replacements::apply_object_variable` and `to_sparql_variable`.

/// Settings payload for `qlueLs/changeSettings` with the given replacements.
///
/// INFO: `changeSettings` merges, so only the changed key has to be sent.
fn settings_with_replacements(replacements: Value) -> Value {
    json!({ "replacements": replacements })
}

#[test]
fn test_object_variable_completion_applies_default_has_replacement() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            .open_document(
                "file:///test.sparql",
                //                     0123456789012345678901234567890123456
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { ?s wdt:hasAuthor ? }",
            )
            .await;

        let id = client.complete("file:///test.sparql", 1, 35).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");
        let labels = get_completion_labels(&response);

        // "hasAuthor" -> "Author" -> "?author"
        assert!(
            labels.contains(&"?author".to_string()),
            "Should suggest ?author for predicate wdt:hasAuthor, got: {:?}",
            labels
        );
        // The subject is a variable, so the combined name is offered too.
        assert!(
            labels.contains(&"?s_author".to_string()),
            "Should suggest ?s_author for subject ?s, got: {:?}",
            labels
        );
        assert!(
            !labels.contains(&"?hasAuthor".to_string()),
            "Should NOT suggest the unreplaced ?hasAuthor, got: {:?}",
            labels
        );
    });
}

#[test]
fn test_object_variable_completion_splits_camel_case_into_snake_case() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            .open_document(
                "file:///test.sparql",
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { ?person wdt:hasBirthDate ? }",
            )
            .await;

        let id = client.complete("file:///test.sparql", 1, 43).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");
        let labels = get_completion_labels(&response);

        // "hasBirthDate" -> "Birth_Date" -> "?birth_date"
        assert!(
            labels.contains(&"?birth_date".to_string()),
            "Should suggest ?birth_date for predicate wdt:hasBirthDate, got: {:?}",
            labels
        );
        assert!(
            labels.contains(&"?person_birth_date".to_string()),
            "Should suggest ?person_birth_date for subject ?person, got: {:?}",
            labels
        );
        assert!(
            !labels.contains(&"?birthdate".to_string()),
            "Should NOT suggest the glued ?birthdate, got: {:?}",
            labels
        );
    });
}

#[test]
fn test_object_variable_completion_applies_default_ed_by_replacement() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            .open_document(
                "file:///test.sparql",
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { ?s wdt:authoredBy ? }",
            )
            .await;

        let id = client.complete("file:///test.sparql", 1, 36).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");
        let labels = get_completion_labels(&response);

        // "authoredBy" -> "author" -> "?author"
        assert!(
            labels.contains(&"?author".to_string()),
            "Should suggest ?author for predicate wdt:authoredBy, got: {:?}",
            labels
        );
    });
}

#[test]
fn test_object_variable_completion_unmatched_predicate_is_lowercased_only() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            .open_document(
                "file:///test.sparql",
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { ?s wdt:Author ? }",
            )
            .await;

        let id = client.complete("file:///test.sparql", 1, 32).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");
        let labels = get_completion_labels(&response);

        // No replacement matches, only the snake_case conversion applies.
        assert!(
            labels.contains(&"?author".to_string()),
            "Should suggest ?author for predicate wdt:Author, got: {:?}",
            labels
        );
    });
}

#[test]
fn test_object_variable_completion_triggered_by_question_mark() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            .open_document(
                "file:///test.sparql",
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { ?s wdt:hasAuthor ? }",
            )
            .await;

        // The "?" trigger character reaches the same code path.
        let id = client
            .complete_triggered("file:///test.sparql", 1, 35, "?")
            .await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");
        let labels = get_completion_labels(&response);

        assert!(
            labels.contains(&"?author".to_string()),
            "Should suggest ?author on \"?\" trigger, got: {:?}",
            labels
        );
    });
}

#[test]
fn test_object_variable_completion_uses_custom_replacements() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            .change_settings(settings_with_replacements(json!({
                "objectVariable": [
                    { "pattern": r"^has(\w+)", "replacement": "my$1" }
                ]
            })))
            .await;

        client
            .open_document(
                "file:///test.sparql",
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { ?s wdt:hasAuthor ? }",
            )
            .await;

        let id = client.complete("file:///test.sparql", 1, 35).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");
        let labels = get_completion_labels(&response);

        // "hasAuthor" -> "myAuthor" -> "?myauthor"
        assert!(
            labels.contains(&"?myauthor".to_string()),
            "Should apply the configured replacement, got: {:?}",
            labels
        );
        assert!(
            !labels.contains(&"?author".to_string()),
            "Should NOT apply the default replacements once overridden, got: {:?}",
            labels
        );
    });
}

#[test]
fn test_object_variable_completion_with_empty_replacements_keeps_local_name() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            .change_settings(settings_with_replacements(json!({
                "objectVariable": []
            })))
            .await;

        client
            .open_document(
                "file:///test.sparql",
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { ?s wdt:hasAuthor ? }",
            )
            .await;

        let id = client.complete("file:///test.sparql", 1, 35).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");
        let labels = get_completion_labels(&response);

        // Only the built in snake_case conversion remains.
        assert!(
            labels.contains(&"?hasauthor".to_string()),
            "Should fall back to the plain local name, got: {:?}",
            labels
        );
    });
}

#[test]
fn test_object_variable_completion_keeps_replacements_on_unrelated_settings_change() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // NOTE: `changeSettings` merges, so a change that does not mention
        // `replacements` leaves the defaults in place.
        client
            .change_settings(json!({ "format": { "alignPredicates": false } }))
            .await;

        client
            .open_document(
                "file:///test.sparql",
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { ?s wdt:hasAuthor ? }",
            )
            .await;

        let id = client.complete("file:///test.sparql", 1, 35).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");
        let labels = get_completion_labels(&response);

        assert!(
            labels.contains(&"?author".to_string()),
            "Should still apply the default replacements, got: {:?}",
            labels
        );
    });
}

#[test]
fn test_object_variable_completion_survives_an_invalid_replacement_pattern() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // WARNING: this used to panic on the next completion request.
        client
            .change_settings(settings_with_replacements(json!({
                "objectVariable": [{ "pattern": "([unclosed", "replacement": "" }]
            })))
            .await;

        client
            .open_document(
                "file:///test.sparql",
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { ?s wdt:hasAuthor ? }",
            )
            .await;

        let id = client.complete("file:///test.sparql", 1, 35).await;
        let response = client
            .get_response(id)
            .expect("Should still receive completion response");
        let labels = get_completion_labels(&response);

        // The rejected change left the previous settings in place.
        assert!(
            labels.contains(&"?author".to_string()),
            "Should keep the previous replacements, got: {:?}",
            labels
        );
    });
}

#[test]
fn test_object_variable_completion_with_replacements_disabled() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // INFO: an explicit null is how replacements are turned off, an omitted
        // key would be kept at its current value.
        client
            .change_settings(json!({ "replacements": null }))
            .await;

        client
            .open_document(
                "file:///test.sparql",
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { ?s wdt:hasAuthor ? }",
            )
            .await;

        let id = client.complete("file:///test.sparql", 1, 35).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");
        let labels = get_completion_labels(&response);

        assert!(
            labels.contains(&"?hasauthor".to_string()),
            "Should suggest the unreplaced local name, got: {:?}",
            labels
        );
    });
}

#[test]
fn test_object_variable_completion_with_non_variable_subject() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            .open_document(
                "file:///test.sparql",
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { <http://a> wdt:hasAuthor ? }",
            )
            .await;

        let id = client.complete("file:///test.sparql", 1, 43).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");
        let labels = get_completion_labels(&response);

        assert!(
            labels.contains(&"?author".to_string()),
            "Should suggest ?author, got: {:?}",
            labels
        );
        // No subject variable, so no combined "?subject_object" suggestion.
        assert!(
            labels.iter().all(|label| !label.ends_with("_author")),
            "Should NOT suggest a combined variable without a subject variable, got: {:?}",
            labels
        );
    });
}

#[test]
fn test_object_variable_completion_in_blank_node() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        client
            .open_document(
                "file:///test.sparql",
                "PREFIX wdt: <http://x/>\nSELECT * WHERE { ?s wdt:p [ wdt:hasAuthor ? ] }",
            )
            .await;

        let id = client.complete("file:///test.sparql", 1, 43).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");
        let labels = get_completion_labels(&response);

        assert!(
            labels.contains(&"?author".to_string()),
            "Should suggest ?author inside a blank node, got: {:?}",
            labels
        );
    });
}

/// A partially typed multi word keyword keeps offering that keyword.
///
/// The first word lexes as its own keyword token, so once a space follows it
/// the anchor parks on GROUP and the only continuation is BY. The location has
/// to stay SolutionModifier for the keyword to still be suggested.
#[test]
fn test_completion_partial_multi_word_keyword() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        //                                       0123456789
        let document = "SELECT * WHERE {\n  ?a ?b ?c .\n}\nGROUP B";
        client.open_document("file:///test.sparql", document).await;

        let id = client.complete("file:///test.sparql", 3, 7).await;
        let response = client
            .get_response(id)
            .expect("Should receive completion response");

        assert!(
            has_completion_label(&response, "GROUP BY"),
            "Should suggest GROUP BY for 'GROUP B', got: {:?}",
            get_completion_labels(&response)
        );
        assert!(
            !has_completion_label(&response, "LIMIT"),
            "Should NOT suggest LIMIT for 'GROUP B'"
        );

        // The edit has to span the whole term, so accepting it does not leave
        // the "GROUP B" the user already typed behind.
        let range = &response["result"]["items"][0]["textEdit"]["range"];
        assert_eq!(range["start"], json!({"line": 3, "character": 0}));
        assert_eq!(range["end"], json!({"line": 3, "character": 7}));
    });
}
