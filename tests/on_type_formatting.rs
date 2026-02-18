//! End-to-end tests for SPARQL on-type formatting
//!
//! Tests the textDocument/onTypeFormatting LSP method with various triggers.

mod harness;

use harness::TestClient;
use harness::runtime::run_lsp_test;
use serde_json::json;

// =============================================================================
// Newline trigger tests (existing functionality)
// =============================================================================

#[test]
fn test_on_type_newline_basic_indent() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Document with a newline inside braces
        // Position is right after the newline (line 1, char 0)
        client
            .open_document(
                "file:///test.sparql",
                //         1         2         3
                //1234567890123456789012345678901234567890
                "SELECT * WHERE {\n?s ?p ?o }",
            )
            .await;

        let id = client
            .on_type_format("file:///test.sparql", 1, 0, "\n")
            .await;
        let response = client.get_response(id).expect("Should receive response");

        // Should have indentation edits
        let result = &response["result"];
        assert!(result.is_array(), "Result should be an array");
        let edits = result.as_array().unwrap();
        assert!(!edits.is_empty(), "Should have formatting edits");

        // The edit should insert proper indentation (2 spaces for depth 1)
        let edit = &edits[0];
        assert_eq!(edit["newText"], "  ", "Should indent with 2 spaces");
    });
}

#[test]
fn test_on_type_newline_after_semicolon() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Document with semicolon followed by newline
        // Position is right after the newline (line 1, char 0)
        client
            .open_document(
                "file:///test.sparql",
                //         1         2         3
                //1234567890123456789012345678901234567890
                "SELECT * WHERE { ?s ?p ?o ;\n?p2 ?o2 }",
            )
            .await;

        let id = client
            .on_type_format("file:///test.sparql", 1, 0, "\n")
            .await;
        let response = client.get_response(id).expect("Should receive response");

        // Should have indentation edits with predicate alignment
        let result = &response["result"];
        assert!(result.is_array(), "Result should be an array");
        let edits = result.as_array().unwrap();
        assert!(!edits.is_empty(), "Should have formatting edits");

        // The edit should align with the first predicate column
        let edit = &edits[0];
        let new_text = edit["newText"].as_str().unwrap();
        // "SELECT * WHERE { ?s " is 20 chars, so predicate starts at column 20
        assert!(
            new_text.len() > 2,
            "Should indent more than base for predicate alignment"
        );
    });
}

// =============================================================================
// Semicolon trigger tests (new functionality)
// =============================================================================

#[test]
fn test_on_type_semicolon_disabled_by_default() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // auto_line_break is disabled by default
        client
            .open_document(
                "file:///test.sparql",
                //         1         2         3
                //1234567890123456789012345678901234567890
                "SELECT * WHERE { ?s ?p ?o; }",
            )
            .await;

        // Position after the semicolon (line 0, char 26)
        let id = client
            .on_type_format("file:///test.sparql", 0, 26, ";")
            .await;
        let response = client.get_response(id).expect("Should receive response");

        // Should return null (no edits)
        assert!(
            response["result"].is_null(),
            "Result should be null when auto_line_break is disabled"
        );
    });
}

#[test]
fn test_on_type_semicolon_with_valid_triple() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Enable auto_line_break
        client
            .change_settings(json!({
                "autoLineBreak": true
            }))
            .await;

        // Valid triple: ?s ?p ?o followed by semicolon
        client
            .open_document(
                "file:///test.sparql",
                //         1         2         3
                //1234567890123456789012345678901234567890
                "SELECT * WHERE { ?s ?p ?o; }",
            )
            .await;

        // Position after the semicolon (line 0, char 26)
        let id = client
            .on_type_format("file:///test.sparql", 0, 26, ";")
            .await;
        let response = client.get_response(id).expect("Should receive response");

        // Debug: print the full response
        eprintln!("Response: {:?}", response);

        // Should have edits to insert newline + indent
        let result = &response["result"];
        assert!(
            result.is_array(),
            "Result should be an array, got: {:?}",
            result
        );
        let edits = result.as_array().unwrap();
        assert!(!edits.is_empty(), "Should have formatting edits");

        // The edit should insert newline + proper indentation
        let edit = &edits[0];
        let new_text = edit["newText"].as_str().unwrap();
        assert!(
            new_text.starts_with('\n'),
            "Should insert newline: got '{}'",
            new_text
        );
    });
}

#[test]
fn test_on_type_semicolon_with_invalid_triple() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Enable auto_line_break
        client
            .change_settings(json!({
                "autoLineBreak": true
            }))
            .await;

        // Invalid triple: ?s ?p (missing object) followed by semicolon
        client
            .open_document(
                "file:///test.sparql",
                //         1         2         3
                //1234567890123456789012345678901234567890
                "SELECT * WHERE { ?s ?p; }",
            )
            .await;

        // Position after the semicolon (line 0, char 23)
        let id = client
            .on_type_format("file:///test.sparql", 0, 23, ";")
            .await;
        let response = client.get_response(id).expect("Should receive response");

        // Should return null (invalid triple)
        assert!(
            response["result"].is_null(),
            "Result should be null for invalid triple"
        );
    });
}

// =============================================================================
// Dot trigger tests (new functionality)
// =============================================================================

#[test]
fn test_on_type_dot_disabled_by_default() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // auto_line_break is disabled by default
        client
            .open_document(
                "file:///test.sparql",
                //         1         2         3
                //1234567890123456789012345678901234567890
                "SELECT * WHERE { ?s ?p ?o. }",
            )
            .await;

        // Position after the dot (line 0, char 26)
        let id = client
            .on_type_format("file:///test.sparql", 0, 26, ".")
            .await;
        let response = client.get_response(id).expect("Should receive response");

        // Should return null (no edits)
        assert!(
            response["result"].is_null(),
            "Result should be null when auto_line_break is disabled"
        );
    });
}

#[test]
fn test_on_type_dot_with_valid_triple() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Enable auto_line_break
        client
            .change_settings(json!({
                "autoLineBreak": true
            }))
            .await;

        // Valid triple: ?s ?p ?o followed by dot
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { ?s ?p ?o. }")
            .await;

        // Position after the dot (line 0, char 26)
        let id = client
            .on_type_format("file:///test.sparql", 0, 26, ".")
            .await;
        let response = client.get_response(id).expect("Should receive response");

        // Should have edits to insert newline + indent
        let result = &response["result"];
        assert!(result.is_array(), "Result should be an array");
        let edits = result.as_array().unwrap();
        assert!(!edits.is_empty(), "Should have formatting edits");

        // The edit should insert newline + base indentation (2 spaces for depth 1)
        let edit = &edits[0];
        let new_text = edit["newText"].as_str().unwrap();
        assert!(
            new_text.starts_with('\n'),
            "Should insert newline: got '{}'",
            new_text
        );
        // For dot, indent should be base level (2 spaces for one brace depth)
        assert_eq!(new_text, "\n  ", "Should indent with newline + 2 spaces");
    });
}

#[test]
fn test_on_type_dot_with_invalid_triple() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Enable auto_line_break
        client
            .change_settings(json!({
                "autoLineBreak": true
            }))
            .await;

        // Invalid triple: ?s ?p (missing object) followed by dot
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { ?s ?p. }")
            .await;

        // Position after the dot (line 0, char 23)
        let id = client
            .on_type_format("file:///test.sparql", 0, 23, ".")
            .await;
        let response = client.get_response(id).expect("Should receive response");

        // Should return null (invalid triple)
        assert!(
            response["result"].is_null(),
            "Result should be null for invalid triple"
        );
    });
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_on_type_semicolon_nested_depth() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Enable auto_line_break
        client
            .change_settings(json!({
                "autoLineBreak": true
            }))
            .await;

        // Nested pattern with OPTIONAL
        client
            .open_document(
                "file:///test.sparql",
                //  0         1         2         3         4
                //  01234567890123456789012345678901234567890
                // "SELECT * WHERE { OPTIONAL { ?s ?p ?o; } }"
                //                                      ^ semicolon at offset 37
                "SELECT * WHERE { OPTIONAL { ?s ?p ?o; } }",
            )
            .await;

        // Position after the semicolon (line 0, char 37)
        let id = client
            .on_type_format("file:///test.sparql", 0, 37, ";")
            .await;
        let response = client.get_response(id).expect("Should receive response");

        // Should have edits with proper nested indentation
        let result = &response["result"];
        assert!(result.is_array(), "Result should be an array");
        let edits = result.as_array().unwrap();
        assert!(!edits.is_empty(), "Should have formatting edits");

        // The edit should account for nested depth
        let edit = &edits[0];
        let new_text = edit["newText"].as_str().unwrap();
        assert!(
            new_text.starts_with('\n'),
            "Should insert newline: got '{}'",
            new_text
        );
    });
}

#[test]
fn test_on_type_dot_nested_depth() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Enable auto_line_break
        client
            .change_settings(json!({
                "autoLineBreak": true
            }))
            .await;

        // Nested pattern with OPTIONAL
        client
            .open_document(
                "file:///test.sparql",
                //         1         2         3
                //1234567890123456789012345678901234567890
                "SELECT * WHERE { OPTIONAL { ?s ?p ?o. } }",
            )
            .await;

        // Position after the dot (line 0, char 35)
        let id = client
            .on_type_format("file:///test.sparql", 0, 37, ".")
            .await;
        let response = client.get_response(id).expect("Should receive response");

        // Should have edits with proper nested indentation
        let result = &response["result"];
        assert!(result.is_array(), "Result should be an array");
        let edits = result.as_array().unwrap();
        assert!(!edits.is_empty(), "Should have formatting edits");

        // The edit should have depth 2 indentation (4 spaces)
        let edit = &edits[0];
        let new_text = edit["newText"].as_str().unwrap();
        assert_eq!(
            new_text, "\n    ",
            "Should indent with newline + 4 spaces for depth 2"
        );
    });
}
