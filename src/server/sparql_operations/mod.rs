#[cfg(not(target_arch = "wasm32"))]
mod native;
mod utils;
#[cfg(target_arch = "wasm32")]
mod wasm;
use crate::server::lsp::CanceledError;
use crate::server::lsp::QLeverException;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native::*;
#[cfg(target_arch = "wasm32")]
pub(crate) use wasm::*;

/// Everything that can go wrong when sending a SPARQL request
#[derive(Debug)]
pub(super) enum SparqlRequestError {
    // NOTE: `Timeout` is only constructed on native, `Canceled` only on wasm.
    /// The request did not complete within the configured time limit.
    #[allow(dead_code)]
    Timeout,
    /// The request was canceled by the client before a response arrived.
    #[allow(dead_code)]
    Canceled(CanceledError),
    /// The http connection to the endpoint could not be established.
    Connection(ConnectionError),
    /// The endpoint responded with a non 2xx status code and no structured error body.
    Http(HttpError),
    /// The response body could not be read or deserialized into the expected shape.
    Deserialization(String),
    /// The endpoint responded with a structured QLever error message.
    QLeverException(QLeverException),
}

/// A non 2xx HTTP response whose body did not contain a structured
/// (engine-specific) error message.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpError {
    pub status: u16,
    pub status_text: String,
    /// Raw response body (may be html, plain text, ...)
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionError {
    pub query: String,
    /// The underlying network error (no http response was received).
    pub message: String,
}
