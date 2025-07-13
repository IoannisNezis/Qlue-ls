use crate::server::{configuration::RequestMethod, tracing::TraceFile};

use super::lsp::{
    errors::{ErrorCode, LSPError},
    textdocument::{TextDocumentItem, TextEdit},
    Backend, TextDocumentContentChangeEvent, TraceValue,
};
use curies::{Converter, CuriesError};
use futures::lock::Mutex;
use ll_sparql_parser::{parse, SyntaxNode};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, PartialEq)]
pub enum ServerStatus {
    Initializing,
    Running,
    ShuttingDown,
}

pub struct ServerState {
    pub status: ServerStatus,
    pub trace_value: TraceValue,
    documents: HashMap<String, TextDocumentItem>,
    backends: HashMap<String, Backend>,
    request_method: HashMap<String, RequestMethod>,
    uri_converter: HashMap<String, Converter>,
    default_backend: Option<String>,
    parse_tree_cache: Option<(String, u32, SyntaxNode)>,
    request_id_counter: u32,
    pub label_memory: HashMap<String, String>,
    pub(super) trace_events: Rc<Mutex<TraceFile>>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            status: ServerStatus::Initializing,
            trace_value: TraceValue::Off,
            documents: HashMap::new(),
            backends: HashMap::new(),
            request_method: HashMap::new(),
            uri_converter: HashMap::new(),
            default_backend: None,
            parse_tree_cache: None,
            request_id_counter: 0,
            label_memory: HashMap::new(),
            trace_events: Rc::new(Mutex::new(TraceFile::default())),
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

    pub(super) fn get_default_backend(&self) -> Option<&Backend> {
        self.backends.get(self.default_backend.as_ref()?)
    }

    pub fn add_backend(&mut self, backend: Backend) {
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

    pub fn get_backend(&self, backend_name: &str) -> Option<&Backend> {
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
        document.apply_text_edits(
            content_changes
                .into_iter()
                .map(TextEdit::from_text_document_content_change_event)
                .collect::<Vec<TextEdit>>(),
        );
        document.increase_version();
        Ok(())
    }

    pub(super) fn get_document(&self, uri: &str) -> Result<&TextDocumentItem, LSPError> {
        self.documents.get(uri).ok_or(LSPError::new(
            ErrorCode::InvalidRequest,
            &format!("Requested document \"{}\"could not be found", uri),
        ))
    }

    pub(super) fn get_cached_parse_tree(&mut self, uri: &str) -> Result<SyntaxNode, LSPError> {
        let document = self.documents.get(uri).ok_or(LSPError::new(
            ErrorCode::InvalidRequest,
            &format!("Requested document \"{}\"could not be found", uri),
        ))?;
        if let Some((cached_uri, cached_version, cached_root)) = self.parse_tree_cache.as_ref() {
            if uri == cached_uri && *cached_version == document.version() {
                return Ok(cached_root.clone());
            }
        }
        let root = parse(&document.text);
        self.parse_tree_cache = Some((uri.to_string(), document.version(), root.clone()));
        Ok(root)
    }

    pub(crate) fn get_default_converter(&self) -> Option<&Converter> {
        self.uri_converter.get(self.default_backend.as_ref()?)
    }

    pub(crate) fn get_converter(&self, backend_name: &str) -> Option<&Converter> {
        self.uri_converter.get(backend_name)
    }

    pub(crate) fn get_all_backends(&self) -> Vec<&Backend> {
        self.backends.values().collect()
    }
}
