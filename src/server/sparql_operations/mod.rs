#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;
use crate::server::lsp::CanceledError;
#[cfg(target_arch = "wasm32")]
use crate::server::lsp::QLeverException;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native::*;
#[cfg(target_arch = "wasm32")]
pub(crate) use wasm::*;

/// Everything that can go wrong when sending a SPARQL request
/// - `Timeout`: The request took to long
/// - `Connection`: The Http connection could not be established
/// - `Response`: The responst had a non 200 status code
/// - `Deserialization`: The response could not be deserialized
///
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub(super) enum SparqlRequestError {
    Timeout,
    Connection(ConnectionError),
    Response(String),
    Deserialization(String),
    QLeverException(QLeverException),
    _Canceled(CanceledError),
}
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub(super) enum SparqlRequestError {
    Timeout,
    Connection(ConnectionError),
    Response(String),
    Deserialization(String),
    _Canceled(CanceledError),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionError {
    pub query: String,
    pub status_text: String,
}
