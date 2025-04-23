use std::collections::HashMap;

use curies::{Converter, CuriesError};
use tree_sitter::Tree;

use super::lsp::{
    errors::{ErrorCode, LSPError},
    textdocument::{TextDocumentItem, TextEdit},
    Backend, TextDocumentContentChangeEvent, TraceValue,
};

#[derive(Debug, PartialEq)]
pub enum ServerStatus {
    Initializing,
    Running,
    ShuttingDown,
}

pub struct ServerState {
    pub status: ServerStatus,
    pub trace_value: TraceValue,
    documents: HashMap<String, (TextDocumentItem, Option<Tree>)>,
    backends: HashMap<String, Backend>,
    uri_converter: HashMap<String, Converter>,
    default_backend: Option<String>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            status: ServerStatus::Initializing,
            trace_value: TraceValue::Off,
            documents: HashMap::new(),
            backends: HashMap::new(),
            uri_converter: HashMap::new(),
            default_backend: None,
        }
    }

    pub fn get_backend_name_by_url(&self, url: &str) -> Option<String> {
        self.backends
            .iter()
            .find_map(|(key, backend)| (backend.url == url).then(|| key.clone()))
    }

    pub fn set_default_backend(&mut self, name: String) {
        self.default_backend = Some(name)
    }

    pub fn get_default_backend(&self) -> Option<&Backend> {
        self.backends.get(self.default_backend.as_ref()?)
    }

    pub fn add_backend(&mut self, backend: Backend) {
        self.backends.insert(backend.name.clone(), backend);
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

    pub(super) fn add_document(&mut self, text_document: TextDocumentItem, tree: Option<Tree>) {
        self.documents
            .insert(text_document.uri.clone(), (text_document, tree));
    }

    pub(super) fn change_document(
        &mut self,
        uri: &String,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<(), LSPError> {
        let document = &mut self
            .documents
            .get_mut(uri)
            .ok_or(LSPError::new(
                ErrorCode::InvalidParams,
                &format!("Could not change unknown document {}", uri),
            ))?
            .0;

        document.apply_text_edits(
            content_changes
                .into_iter()
                .map(TextEdit::from_text_document_content_change_event)
                .collect::<Vec<TextEdit>>(),
        );
        Ok(())
    }

    pub(super) fn get_state(&self, uri: &str) -> Result<(&TextDocumentItem, &Tree), LSPError> {
        match self.documents.get(uri) {
            Some((document, Some(tree))) => Ok((document, tree)),
            Some((_document, None)) => Err(LSPError::new(
                ErrorCode::InternalError,
                &format!("Could not find parse-tree for \"{}\"", uri),
            )),
            None => Err(LSPError::new(
                ErrorCode::InternalError,
                &format!("Could not find document \"{}\"", uri),
            )),
        }
    }

    pub(super) fn get_document(&self, uri: &str) -> Result<&TextDocumentItem, LSPError> {
        Ok(&self
            .documents
            .get(uri)
            .ok_or(LSPError::new(
                ErrorCode::InvalidRequest,
                &format!("Requested document \"{}\"could not be found", uri),
            ))?
            .0)
    }

    pub(super) fn update_tree(
        &mut self,
        uri: &str,
        new_tree: Option<Tree>,
    ) -> Result<(), LSPError> {
        if self.documents.get(uri).is_some() {
            self.documents
                .entry(uri.to_string())
                .and_modify(|(_document, old_tree)| *old_tree = new_tree);
            Ok(())
        } else {
            Err(LSPError::new(
                ErrorCode::InternalError,
                &format!("Could not update parse-tree, no entry for \"{}\"", uri),
            ))
        }
    }

    pub(crate) fn get_default_converter(&self) -> Option<&Converter> {
        self.uri_converter.get((&self.default_backend).as_ref()?)
    }

    pub(crate) fn get_converter(&self, backend_name: &str) -> Option<&Converter> {
        self.uri_converter.get(backend_name)
    }

    pub(crate) fn get_all_backends(&self) -> Vec<&Backend> {
        self.backends.values().collect()
    }
}
