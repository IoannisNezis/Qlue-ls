//! Stub implementations of the SPARQL-over-HTTP functions for builds without
//! the `http-client` feature. Every function immediately returns an error (or
//! `false` for the health check) so the crate compiles and links without
//! `reqwest` and its transitive TLS dependencies.
//!
//! This exists so that consumers who embed qlue-ls purely as a standalone
//! SPARQL formatter or parser (i.e. they never execute queries against a remote
//! endpoint) can build with `default-features = false` and avoid linking the
//! macOS Security/SystemConfiguration/CoreFoundation frameworks that `reqwest`'s
//! TLS stack requires — which in turn enables cross-compilation to
//! `*-apple-darwin` targets from Linux builders without a macOS SDK.

use crate::server::Server;
use crate::server::configuration::BackendConfiguration;
use crate::server::configuration::RequestMethod;
use crate::server::lsp::ExecuteUpdateResponseResult;
use crate::server::lsp::SparqlEngine;
use crate::server::sparql_operations::ConnectionError;
use crate::server::sparql_operations::SparqlRequestError;
use crate::sparql::results::SparqlResult;
use futures::lock::Mutex;
use std::rc::Rc;

const NO_HTTP: &str = "qlue-ls was compiled without the \"http-client\" feature; \
                        SPARQL endpoint communication is not available";

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_query(
    _server_rc: Rc<Mutex<Server>>,
    _url: String,
    query: String,
    _query_id: Option<&str>,
    _engine: Option<SparqlEngine>,
    _timeout_ms: Option<u32>,
    _method: RequestMethod,
    _limit: Option<usize>,
    _offset: usize,
    _lazy: bool,
) -> Result<Option<SparqlResult>, SparqlRequestError> {
    Err(SparqlRequestError::Connection(ConnectionError {
        message: NO_HTTP.to_string(),
        query,
    }))
}

pub(crate) async fn check_server_availability(_backend: &BackendConfiguration) -> bool {
    false
}

pub(crate) async fn execute_construct_query(
    _server_rc: Rc<Mutex<Server>>,
    _url: &str,
    _query: &str,
    _query_id: Option<&str>,
    _engine: Option<SparqlEngine>,
    _lazy: bool,
) -> Result<Option<SparqlResult>, SparqlRequestError> {
    Err(SparqlRequestError::Connection(ConnectionError {
        message: NO_HTTP.to_string(),
        query: String::new(),
    }))
}

pub(crate) async fn execute_update(
    _server_rc: Rc<Mutex<Server>>,
    _url: &str,
    _query: &str,
    _query_id: Option<&str>,
    _access_token: Option<&str>,
) -> Result<ExecuteUpdateResponseResult, SparqlRequestError> {
    Err(SparqlRequestError::Connection(ConnectionError {
        message: NO_HTTP.to_string(),
        query: String::new(),
    }))
}
