mod harness;

use harness::TestClient;
use harness::runtime::run_lsp_test;
use serde_json::Value;

#[test]
fn test_parse_tree_returns_root_node() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { }")
            .await;

        let id = client.parse_tree("file:///test.sparql").await;
        let response = client
            .get_response(id)
            .expect("Should receive parse tree response");

        let result = &response["result"];
        assert!(result["timeMs"].is_f64(), "Response should include timeMs");
        let tree = &result["tree"];
        assert_eq!(tree["type"], "node", "Root should be a node");
        assert!(tree["kind"].is_string(), "Root should have a kind");
        assert!(
            tree["children"].is_array(),
            "Root node should have children"
        );
        assert!(tree["range"].is_object(), "Root should have a range");
    });
}

#[test]
fn test_parse_tree_root_range_spans_document() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document("file:///test.sparql", "SELECT *\nWHERE { }")
            .await;

        let id = client.parse_tree("file:///test.sparql").await;
        let response = client.get_response(id).unwrap();
        let range = &response["result"]["tree"]["range"];

        assert_eq!(range["start"]["line"], 0);
        assert_eq!(range["start"]["character"], 0);
        assert_eq!(range["end"]["line"], 1);
        assert_eq!(range["end"]["character"], 9);
    });
}

#[test]
fn test_parse_tree_contains_tokens_with_text() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document("file:///test.sparql", "SELECT *")
            .await;

        let id = client.parse_tree("file:///test.sparql").await;
        let response = client.get_response(id).unwrap();

        let tokens = collect_tokens(&response["result"]["tree"]);
        let texts: Vec<&str> = tokens.iter().filter_map(|t| t["text"].as_str()).collect();

        assert!(
            texts.contains(&"\"SELECT\""),
            "Should contain SELECT token, got: {:?}",
            texts
        );
    });
}

#[test]
fn test_parse_tree_tokens_have_no_children() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document("file:///test.sparql", "SELECT *")
            .await;

        let id = client.parse_tree("file:///test.sparql").await;
        let response = client.get_response(id).unwrap();

        let tokens = collect_tokens(&response["result"]["tree"]);
        for token in &tokens {
            assert!(
                token.get("children").is_none(),
                "Tokens should not have children: {:?}",
                token
            );
        }
    });
}

#[test]
fn test_parse_tree_nodes_have_no_text() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { ?s ?p ?o }")
            .await;

        let id = client.parse_tree("file:///test.sparql").await;
        let response = client.get_response(id).unwrap();

        let nodes = collect_nodes(&response["result"]["tree"]);
        for node in &nodes {
            assert!(
                node.get("text").is_none(),
                "Nodes should not have text: {:?}",
                node
            );
        }
    });
}

#[test]
fn test_parse_tree_with_prefix() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document(
                "file:///test.sparql",
                "PREFIX ex: <http://example.org/>\nSELECT * WHERE { ?s ex:p ?o }",
            )
            .await;

        let id = client.parse_tree("file:///test.sparql").await;
        let response = client.get_response(id).unwrap();
        let tree = &response["result"]["tree"];

        assert_eq!(tree["type"], "node");

        let tokens = collect_tokens(tree);
        let texts: Vec<&str> = tokens.iter().filter_map(|t| t["text"].as_str()).collect();
        assert!(texts.contains(&"\"PREFIX\""), "Should contain PREFIX token");
        assert!(texts.contains(&"\"SELECT\""), "Should contain SELECT token");
    });
}

#[test]
fn test_parse_tree_empty_document() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client.open_document("file:///test.sparql", "").await;

        let id = client.parse_tree("file:///test.sparql").await;
        let response = client
            .get_response(id)
            .expect("Should handle empty documents");
        let tree = &response["result"]["tree"];

        assert_eq!(tree["type"], "node");
        assert!(tree["children"].is_array());
    });
}

#[test]
fn test_parse_tree_invalid_document() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document("file:///test.sparql", "NOT VALID SPARQL !!!")
            .await;

        let id = client.parse_tree("file:///test.sparql").await;
        let response = client
            .get_response(id)
            .expect("Should handle invalid documents");
        let tree = &response["result"]["tree"];

        // NOTE: the parser is resilient — it should still produce a tree
        assert_eq!(tree["type"], "node");
        assert!(tree["children"].is_array());
    });
}

#[test]
fn test_parse_tree_unknown_document_returns_error() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        let id = client.parse_tree("file:///nonexistent.sparql").await;
        let response = client.get_response(id).expect("Should receive a response");

        assert!(
            response.get("error").is_some(),
            "Should return an error for unknown document: {:?}",
            response
        );
    });
}

#[test]
fn test_parse_tree_multiline_ranges() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document("file:///test.sparql", "SELECT *\nWHERE {\n  ?s ?p ?o\n}")
            .await;

        let id = client.parse_tree("file:///test.sparql").await;
        let response = client.get_response(id).unwrap();
        let tree = &response["result"]["tree"];

        // NOTE: root range should span from start to end of document
        let range = &tree["range"];
        assert_eq!(range["start"]["line"], 0);
        assert_eq!(range["start"]["character"], 0);
        assert_eq!(range["end"]["line"], 3);
        assert_eq!(range["end"]["character"], 1);

        // NOTE: all tokens should have valid ranges with line >= 0
        let tokens = collect_tokens(tree);
        for token in &tokens {
            let start_line = token["range"]["start"]["line"].as_u64().unwrap();
            let end_line = token["range"]["end"]["line"].as_u64().unwrap();
            assert!(
                end_line >= start_line,
                "Token end line should be >= start line: {:?}",
                token
            );
        }
    });
}

#[test]
fn test_parse_tree_skip_trivia_excludes_whitespace() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { }")
            .await;

        let id = client.parse_tree_with("file:///test.sparql", true).await;
        let response = client.get_response(id).unwrap();

        let tokens = collect_tokens(&response["result"]["tree"]);
        let kinds: Vec<&str> = tokens.iter().filter_map(|t| t["kind"].as_str()).collect();
        assert!(
            !kinds.contains(&"WHITESPACE"),
            "skipTrivia should exclude WHITESPACE tokens, got: {:?}",
            kinds
        );
    });
}

#[test]
fn test_parse_tree_skip_trivia_excludes_comments() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document("file:///test.sparql", "# comment\nSELECT *")
            .await;

        let id = client.parse_tree_with("file:///test.sparql", true).await;
        let response = client.get_response(id).unwrap();

        let tokens = collect_tokens(&response["result"]["tree"]);
        let kinds: Vec<&str> = tokens.iter().filter_map(|t| t["kind"].as_str()).collect();
        assert!(
            !kinds.contains(&"Comment"),
            "skipTrivia should exclude Comment tokens, got: {:?}",
            kinds
        );
    });
}

#[test]
fn test_parse_tree_skip_trivia_false_includes_whitespace() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { }")
            .await;

        let id = client.parse_tree_with("file:///test.sparql", false).await;
        let response = client.get_response(id).unwrap();

        let tokens = collect_tokens(&response["result"]["tree"]);
        let kinds: Vec<&str> = tokens.iter().filter_map(|t| t["kind"].as_str()).collect();
        assert!(
            kinds.contains(&"WHITESPACE"),
            "skipTrivia=false should include WHITESPACE tokens, got: {:?}",
            kinds
        );
    });
}

#[test]
fn test_parse_tree_default_includes_whitespace() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { }")
            .await;

        // NOTE: omitting skipTrivia should default to false (include trivia)
        let id = client.parse_tree("file:///test.sparql").await;
        let response = client.get_response(id).unwrap();

        let tokens = collect_tokens(&response["result"]["tree"]);
        let kinds: Vec<&str> = tokens.iter().filter_map(|t| t["kind"].as_str()).collect();
        assert!(
            kinds.contains(&"WHITESPACE"),
            "Default (no skipTrivia) should include WHITESPACE tokens, got: {:?}",
            kinds
        );
    });
}

#[test]
fn test_parse_tree_crlf_line_endings() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        let lf_text = "SELECT *\nWHERE {\n  ?s ?p ?o\n}";
        let crlf_text = "SELECT *\r\nWHERE {\r\n  ?s ?p ?o\r\n}";

        client.open_document("file:///lf.sparql", lf_text).await;
        client.open_document("file:///crlf.sparql", crlf_text).await;

        let lf_id = client.parse_tree("file:///lf.sparql").await;
        let crlf_id = client.parse_tree("file:///crlf.sparql").await;
        let lf_response = client.get_response(lf_id).unwrap();
        let crlf_response = client.get_response(crlf_id).unwrap();

        // NOTE: root ranges should be identical regardless of line ending style
        assert_eq!(
            lf_response["result"]["tree"]["range"], crlf_response["result"]["tree"]["range"],
            "Root range should be the same for LF and CRLF"
        );

        // NOTE: all token ranges should match between the two documents
        let lf_tokens = collect_tokens(&lf_response["result"]["tree"]);
        let crlf_tokens = collect_tokens(&crlf_response["result"]["tree"]);
        assert_eq!(lf_tokens.len(), crlf_tokens.len());
        for (lf_tok, crlf_tok) in lf_tokens.iter().zip(crlf_tokens.iter()) {
            assert_eq!(
                lf_tok["range"], crlf_tok["range"],
                "Token ranges should match:\n  LF:   {}\n  CRLF: {}",
                lf_tok, crlf_tok
            );
        }
    });
}

/// Recursively collect all token elements from a parse tree
fn collect_tokens(element: &Value) -> Vec<Value> {
    let mut tokens = Vec::new();
    match element["type"].as_str() {
        Some("token") => tokens.push(element.clone()),
        Some("node") => {
            if let Some(children) = element["children"].as_array() {
                for child in children {
                    tokens.extend(collect_tokens(child));
                }
            }
        }
        _ => {}
    }
    tokens
}

/// Recursively collect all node elements from a parse tree
fn collect_nodes(element: &Value) -> Vec<Value> {
    let mut nodes = Vec::new();
    if element["type"].as_str() == Some("node") {
        nodes.push(element.clone());
        if let Some(children) = element["children"].as_array() {
            for child in children {
                nodes.extend(collect_nodes(child));
            }
        }
    }
    nodes
}
