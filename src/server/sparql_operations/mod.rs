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

#[derive(Debug)]
pub struct Window {
    window_size: u32,
    window_offset: u32,
}

impl Window {
    pub fn new(window_size: u32, window_offset: u32) -> Self {
        Self {
            window_size,
            window_offset,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn rewrite(&self, query: &str) -> Option<String> {
        use ll_sparql_parser::{
            ast::{AstNode, QueryUnit},
            parse_query,
        };

        let syntax_tree = QueryUnit::cast(parse_query(query))?;
        let select_query = syntax_tree.select_query()?;
        Some(format!(
            "{}{}{}",
            &query[0..select_query.syntax().text_range().start().into()],
            format!(
                "SELECT * WHERE {{\n{}\n}}\nLIMIT {}\nOFFSET {}",
                select_query.text(),
                self.window_size,
                self.window_offset
            ),
            &query[select_query.syntax().text_range().end().into()..]
        ))
    }
}
