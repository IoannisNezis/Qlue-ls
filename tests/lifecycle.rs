//! End-to-end tests for LSP lifecycle operations
//!
//! Tests the initialize, initialized, shutdown, and exit flow.

mod harness;

use harness::runtime::run_lsp_test;
use harness::TestClient;
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn test_initialize_returns_capabilities() {
    run_lsp_test(|| async {
        let client = TestClient::new();

        let id = client
            .send_request(
                "initialize",
                json!({
                    "processId": 1234,
                    "clientInfo": {
                        "name": "test-client",
                        "version": "1.0.0"
                    },
                    "capabilities": {},
                    "rootUri": "file:///workspace"
                }),
            )
            .await;

        let response = client.get_response(id).expect("Should receive initialize response");

        // Verify we got a result, not an error
        assert!(
            response.get("result").is_some(),
            "Initialize should return a result: {:?}",
            response
        );

        let capabilities = &response["result"]["capabilities"];

        // Check essential capabilities are present
        assert!(
            capabilities.get("textDocumentSync").is_some(),
            "Should have textDocumentSync capability"
        );
        assert!(
            capabilities.get("completionProvider").is_some(),
            "Should have completionProvider capability"
        );
        assert!(
            capabilities.get("documentFormattingProvider").is_some(),
            "Should have documentFormattingProvider capability"
        );

        // Check server info
        assert_eq!(
            response["result"]["serverInfo"]["name"],
            "Qlue-ls",
            "Server name should be Qlue-ls"
        );
    });
}

#[test]
fn test_full_lifecycle() {
    run_lsp_test(|| async {
        let client = TestClient::new();

        // Initialize
        let init_id = client.initialize().await;
        let init_response = client.get_response(init_id).expect("Should receive init response");
        assert!(
            init_response["result"]["capabilities"].is_object(),
            "Should return capabilities"
        );

        // Open a document
        client
            .open_document("file:///test.sparql", "SELECT * WHERE { ?s ?p ?o }")
            .await;

        // Request formatting (verify server is operational)
        let format_id = client.format("file:///test.sparql").await;
        let format_response = client
            .get_response(format_id)
            .expect("Should receive format response");
        assert!(
            format_response["result"].is_array(),
            "Formatting should return an array of edits"
        );

        // Shutdown
        let shutdown_id = client.shutdown().await;
        let shutdown_response = client
            .get_response(shutdown_id)
            .expect("Should receive shutdown response");

        // Shutdown should return null result (not an error)
        assert!(
            shutdown_response.get("error").is_none(),
            "Shutdown should not return an error"
        );
    });
}

#[test]
fn test_document_sync() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Open a document
        let mut doc = client.open("file:///test.sparql", "SELECT * WHERE { }").await;

        // Verify we can request operations on it
        let format_id = doc.format().await;
        let response = client.get_response(format_id).expect("Should get format response");
        assert!(response["result"].is_array());

        // Change the document
        doc.change("SELECT ?a ?b WHERE { ?a ?b ?c }").await;

        // Request formatting again
        let format_id2 = doc.format().await;
        let response2 = client.get_response(format_id2).expect("Should get format response");
        assert!(response2["result"].is_array());

        // Close the document
        doc.close().await;
    });
}

#[test]
fn test_multiple_documents() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Open multiple documents
        client
            .open_document("file:///doc1.sparql", "SELECT * WHERE { ?a ?b ?c }")
            .await;
        client
            .open_document("file:///doc2.sparql", "ASK WHERE { ?x ?y ?z }")
            .await;

        // Request formatting for each
        let format1_id = client.format("file:///doc1.sparql").await;
        let format2_id = client.format("file:///doc2.sparql").await;

        let response1 = client.get_response(format1_id).expect("Should get response for doc1");
        let response2 = client.get_response(format2_id).expect("Should get response for doc2");

        assert!(response1["result"].is_array());
        assert!(response2["result"].is_array());
    });
}

#[test]
fn test_unknown_method_returns_error() {
    run_lsp_test(|| async {
        let client = TestClient::new();
        client.initialize().await;

        // Send an unknown method
        let id = client
            .send_request("unknownMethod/doesNotExist", json!({}))
            .await;

        let response = client.get_response(id).expect("Should receive a response");

        // Should return a method not found error
        assert!(
            response.get("error").is_some(),
            "Unknown method should return an error: {:?}",
            response
        );
    });
}
