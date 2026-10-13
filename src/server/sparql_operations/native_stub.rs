//! Stub implementations of the SPARQL-over-HTTP functions for builds without
//! the `http-client` feature. Every function immediately returns an error (or
//! `false` for the health check) so the crate compiles without `reqwest`.
//! See the `[features]` section in `Cargo.toml` for why the feature exists.

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

pub(crate) async fn check_server_availability(backend: &BackendConfiguration) -> bool {
    tracing::warn!("health check for \"{}\" skipped: {}", backend.url, NO_HTTP);
    false
}

pub(crate) async fn execute_construct_query(
    _server_rc: Rc<Mutex<Server>>,
    _url: &str,
    query: &str,
    _query_id: Option<&str>,
    _engine: Option<SparqlEngine>,
    _lazy: bool,
) -> Result<Option<SparqlResult>, SparqlRequestError> {
    Err(SparqlRequestError::Connection(ConnectionError {
        message: NO_HTTP.to_string(),
        query: query.to_string(),
    }))
}

pub(crate) async fn execute_update(
    _server_rc: Rc<Mutex<Server>>,
    _url: &str,
    query: &str,
    _query_id: Option<&str>,
    _access_token: Option<&str>,
) -> Result<ExecuteUpdateResponseResult, SparqlRequestError> {
    Err(SparqlRequestError::Connection(ConnectionError {
        message: NO_HTTP.to_string(),
        query: query.to_string(),
    }))
}
