//! End-to-end tests for hovering IRIs
//!
//! Hovering a prefixed name renders the backend's `hover` template and sends the
//! resulting query to the backend. These tests run a mock SPARQL endpoint and check
//! the PREFIX declaration in the query it receives: the URI prefix declared in the
//! document wins, the backend's prefix map is only a fallback, and the declared
//! prefix is always the one written in the document.

mod harness;

use std::time::Duration;

use harness::TestClient;
use harness::runtime::run_lsp_test;
use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const HOVER_TEMPLATE: &str = r#"{% include "prefix_declarations" %}
SELECT ?qls_label WHERE { {{ entity }} <http://www.w3.org/2000/01/rdf-schema#label> ?qls_label } LIMIT 1"#;

const LABEL: &str = "some label";

/// Starts a mock SPARQL endpoint that answers every query with `LABEL`.
async fn start_endpoint() -> MockServer {
    let endpoint = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sparql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "head": { "vars": ["qls_label"] },
            "results": {
                "bindings": [{ "qls_label": { "type": "literal", "value": LABEL } }]
            }
        })))
        .mount(&endpoint)
        .await;
    endpoint
}

/// NOTE: Hover is handled in a spawned task, so the response arrives asynchronously.
async fn wait_for_response(client: &TestClient, id: u32) -> Value {
    for _ in 0..500 {
        if let Some(response) = client.get_response(id) {
            return response;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("No response to request {id} within 5s");
}

/// Hovers `text` at `line`/`character` with a backend using `prefix_map` and
/// returns the query the backend received.
async fn hover_query(prefix_map: Value, text: &str, line: u32, character: u32) -> String {
    let endpoint = start_endpoint().await;
    let client = TestClient::new();
    client.initialize().await;
    client
        .add_backend_with(json!({
            "name": "test",
            "url": format!("{}/sparql", endpoint.uri()),
            "default": true,
            "prefixMap": prefix_map,
            "queries": { "hover": HOVER_TEMPLATE }
        }))
        .await;
    client.open_document("file:///test.sparql", text).await;

    let id = client.hover("file:///test.sparql", line, character).await;
    let response = wait_for_response(&client, id).await;
    assert_eq!(
        response["result"]["contents"]["value"], LABEL,
        "Hover should show the label returned by the backend: {response}"
    );

    let requests = endpoint
        .received_requests()
        .await
        .expect("wiremock records requests by default");
    assert_eq!(requests.len(), 1, "Hover should send exactly one query");
    requests[0]
        .url
        .query_pairs()
        .find(|(key, _)| key == "query")
        .map(|(_, query)| query.into_owned())
        .expect("GET request should carry the query as `query` parameter")
}

#[test]
fn hover_declares_synonym_prefix_as_written() {
    run_lsp_test(|| async {
        // NOTE: `schema` and `sdo` share a URI prefix, `schema` becomes the primary
        //       prefix. The query uses the synonym `sdo`, so `sdo` must be declared.
        let query = hover_query(
            json!({
                "schema": "https://schema.org/",
                "sdo": "https://schema.org/"
            }),
            //123456789012345678901234
            "SELECT * WHERE { sdo:name ?p ?o }",
            0,
            22,
        )
        .await;

        assert!(
            query.contains("PREFIX sdo: <https://schema.org/>"),
            "Hover query should declare the prefix as written in the document:\n{query}"
        );
    });
}

#[test]
fn hover_prefers_prefix_declared_in_document() {
    run_lsp_test(|| async {
        let query = hover_query(
            json!({ "ex": "http://backend.example.org/" }),
            //                                          0123456789012345678901
            "PREFIX ex: <http://document.example.org/>\nSELECT * WHERE { ex:thing ?p ?o }",
            1,
            21,
        )
        .await;

        assert!(
            query.contains("PREFIX ex: <http://document.example.org/>"),
            "Hover query should use the URI prefix declared in the document:\n{query}"
        );
        assert!(
            !query.contains("http://backend.example.org/"),
            "Hover query should not use the backend's URI prefix when the document declares one:\n{query}"
        );
    });
}

#[test]
fn hover_uses_prefix_only_declared_in_document() {
    run_lsp_test(|| async {
        let query = hover_query(
            json!({}),
            //                                          0123456789012345678901
            "PREFIX ex: <http://document.example.org/>\nSELECT * WHERE { ex:thing ?p ?o }",
            1,
            21,
        )
        .await;

        assert!(
            query.contains("PREFIX ex: <http://document.example.org/>"),
            "Hover query should declare a prefix the backend does not know:\n{query}"
        );
    });
}

#[test]
fn hover_uses_empty_prefix_declared_in_document() {
    run_lsp_test(|| async {
        let query = hover_query(
            json!({ "ex": "http://backend.example.org/" }),
            //                                        0123456789012345678901
            "PREFIX : <http://document.example.org/>\nSELECT * WHERE { :thing ?p ?o }",
            1,
            20,
        )
        .await;

        assert!(
            query.contains("PREFIX : <http://document.example.org/>"),
            "Hover query should declare the empty prefix from the document:\n{query}"
        );
    });
}

#[test]
fn hover_falls_back_to_backend_prefix_map() {
    run_lsp_test(|| async {
        let query = hover_query(
            json!({ "ex": "http://backend.example.org/" }),
            //123456789012345678901
            "SELECT * WHERE { ex:thing ?p ?o }",
            0,
            21,
        )
        .await;

        assert!(
            query.contains("PREFIX ex: <http://backend.example.org/>"),
            "Hover query should fall back to the backend's prefix map:\n{query}"
        );
    });
}
