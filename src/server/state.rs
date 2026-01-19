//! Server state management and document storage.
//!
//! This module manages all mutable state for the language server, including open
//! documents, registered backends, and cached parse trees.
//!
//! # Key Types
//!
//! - [`ServerState`]: Central state container accessed via `Server.state`
//! - [`ServerStatus`]: Lifecycle state (Initializing, Running, ShuttingDown)
//!
//! # State Components
//!
//! - **Documents**: Open text documents keyed by URI, with incremental sync support
//! - **Backends**: SPARQL endpoints with associated prefix maps and request methods
//! - **Parse tree cache**: Single-entry cache to avoid re-parsing unchanged documents
//! - **URI converters**: CURIE/prefix converters for URI compression per backend
//!
//! # Parse Tree Caching
//!
//! [`ServerState::get_cached_parse_tree`] returns cached parse results when the
//! document URI and version match, avoiding expensive re-parsing for repeated
//! operations on the same document state.
//!
//! # Related Modules
//!
//! - [`super::Server`]: Owns the `ServerState` instance
//! - [`super::lsp::textdocument`]: `TextDocumentItem` stored in documents map

use crate::server::configuration::RequestMethod;

use super::lsp::{
    BackendService, TextDocumentContentChangeEvent,
    errors::{ErrorCode, LSPError},
    textdocument::TextDocumentItem,
};
use curies::{Converter, CuriesError};
use ll_sparql_parser::{SyntaxNode, parse};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum ServerStatus {
    Initializing,
    Running,
    ShuttingDown,
}

#[derive(Debug)]
pub enum ClientType {
    Monaco,
    Neovim,
}

pub struct ServerState {
    pub status: ServerStatus,
    pub client_type: Option<ClientType>,
    documents: HashMap<String, TextDocumentItem>,
    backends: HashMap<String, BackendService>,
    request_method: HashMap<String, RequestMethod>,
    uri_converter: HashMap<String, Converter>,
    default_backend: Option<String>,
    parse_tree_cache: RefCell<Option<(String, u32, SyntaxNode)>>,
    request_id_counter: u32,
    running_requests: HashMap<String, Box<dyn Fn()>>,
    pub label_memory: HashMap<String, String>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            status: ServerStatus::Initializing,
            client_type: None,
            documents: HashMap::new(),
            backends: HashMap::new(),
            request_method: HashMap::new(),
            uri_converter: HashMap::new(),
            default_backend: None,
            parse_tree_cache: RefCell::new(None),
            request_id_counter: 0,
            running_requests: HashMap::new(),
            label_memory: HashMap::new(),
        }
    }

    pub fn bump_request_id(&mut self) -> u32 {
        let current_id = self.request_id_counter;
        self.request_id_counter += 1;
        current_id
    }

    pub fn get_backend_name_by_url(&self, url: &str) -> Option<String> {
        self.backends
            .iter()
            .find_map(|(key, backend)| (backend.url == url).then(|| key.clone()))
    }

    pub fn set_default_backend(&mut self, name: String) {
        self.default_backend = Some(name)
    }

    pub(super) fn get_default_backend(&self) -> Option<&BackendService> {
        self.backends.get(self.default_backend.as_ref()?)
    }

    pub fn add_backend(&mut self, backend: BackendService) {
        self.backends.insert(backend.name.clone(), backend);
    }

    pub fn add_backend_request_method(&mut self, backend: &str, method: RequestMethod) {
        self.request_method.insert(backend.to_string(), method);
    }

    /// Return the configured request method for given backend.
    /// Defaults to `GET`.
    pub fn get_backend_request_method(&self, backend: &str) -> RequestMethod {
        self.request_method
            .get(backend)
            .cloned()
            .unwrap_or(RequestMethod::GET)
    }

    pub async fn add_prefix_map(
        &mut self,
        backend: String,
        map: HashMap<String, String>,
    ) -> Result<(), CuriesError> {
        self.uri_converter
            .insert(backend, Converter::from_prefix_map(map).await?);
        Ok(())
    }

    #[cfg(test)]
    pub fn add_prefix_map_test(
        &mut self,
        backend: String,
        map: HashMap<String, String>,
    ) -> Result<(), CuriesError> {
        let mut converter = Converter::new(":");
        for (prefix, uri_prefix) in map.iter() {
            converter.add_prefix(prefix, uri_prefix)?;
        }
        self.uri_converter.insert(backend, converter);
        Ok(())
    }

    pub fn get_backend(&self, backend_name: &str) -> Option<&BackendService> {
        self.backends.get(backend_name)
    }

    pub(super) fn add_document(&mut self, text_document: TextDocumentItem) {
        self.documents
            .insert(text_document.uri.clone(), text_document);
    }

    pub(super) fn change_document(
        &mut self,
        uri: &String,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<(), LSPError> {
        let document = self.documents.get_mut(uri).ok_or(LSPError::new(
            ErrorCode::InvalidParams,
            &format!("Could not change unknown document {}", uri),
        ))?;
        document.apply_content_changes(content_changes);
        document.increase_version();
        Ok(())
    }

    pub(super) fn get_document(&self, uri: &str) -> Result<&TextDocumentItem, LSPError> {
        self.documents.get(uri).ok_or(LSPError::new(
            ErrorCode::InvalidRequest,
            &format!("Requested document \"{}\"could not be found", uri),
        ))
    }

    pub(super) fn get_cached_parse_tree(&self, uri: &str) -> Result<SyntaxNode, LSPError> {
        let document = self.documents.get(uri).ok_or(LSPError::new(
            ErrorCode::InvalidRequest,
            &format!("Requested document \"{}\"could not be found", uri),
        ))?;
        if let Some((cached_uri, cached_version, cached_root)) =
            self.parse_tree_cache.borrow().as_ref()
        {
            if uri == cached_uri && *cached_version == document.version() {
                return Ok(cached_root.clone());
            }
        }
        let root = parse(&document.text);
        *self.parse_tree_cache.borrow_mut() =
            Some((uri.to_string(), document.version(), root.clone()));
        Ok(root)
    }

    pub(crate) fn get_default_converter(&self) -> Option<&Converter> {
        self.uri_converter.get(self.default_backend.as_ref()?)
    }

    pub(crate) fn get_converter(&self, backend_name: &str) -> Option<&Converter> {
        self.uri_converter.get(backend_name)
    }

    pub(crate) fn get_all_backends(&self) -> Vec<&BackendService> {
        self.backends.values().collect()
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn add_running_request(&mut self, id: String, cancel_fn: Box<dyn Fn()>) {
        self.running_requests.insert(id, cancel_fn);
    }

    pub(crate) fn get_running_request(&mut self, id: &str) -> Option<&Box<dyn Fn()>> {
        self.running_requests.get(id)
    }
}
