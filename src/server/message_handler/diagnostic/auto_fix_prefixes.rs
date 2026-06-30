//! Automatically applies prefix-related fixes by pushing a `workspace/applyEdit`
//! request to the client, without the user having to invoke a code action.
//!
//! Based on the computed diagnostics and the user's settings, this declares
//! prefixes that are used but missing (`undeclared_prefix`) and removes prefix
//! declarations that are never used (`unused_prefix_declaration`).

use super::{undeclared_prefix, unused_prefix_declaration};
use crate::server::{
    Server,
    lsp::{
        DiagnosticRequest, WorkspaceEditRequest, base_types::LSPAny, diagnostic::Diagnostic,
    },
    message_handler::code_action::{declare_prefix, remove_prefix_declaration},
};
use std::{
    collections::{HashMap, HashSet},
    convert::identity,
};

pub(super) fn auto_fix_prefixes(
    server: &mut Server,
    request: &DiagnosticRequest,
    diagnostics: &[Diagnostic],
) {
    let document_uri = request.params.text_document.uri.clone();
    let mut prefixes = HashSet::<&str>::new();
    let edits: Vec<_> = diagnostics
        .iter()
        .filter_map(|diagnostic| {
            if let Some(LSPAny::String(prefix)) = diagnostic.data.as_ref() {
                if prefixes.insert(prefix) {
                    match diagnostic.code.as_ref() {
                        Some(code)
                            if code == &*undeclared_prefix::CODE
                                && server.settings.prefixes.as_ref().is_some_and(|prefixes| {
                                    prefixes.add_missing.is_some_and(identity)
                                }) =>
                        {
                            declare_prefix(server, &document_uri, diagnostic.clone())
                        }
                        Some(code)
                            if code == &*unused_prefix_declaration::CODE
                                && server.settings.prefixes.as_ref().is_some_and(|prefixes| {
                                    prefixes.remove_unused.is_some_and(identity)
                                }) =>
                        {
                            remove_prefix_declaration(server, &document_uri, diagnostic.clone())
                        }
                        _ => Ok(None),
                    }
                    .ok()
                    .flatten()
                    .and_then(|code_action| code_action.edit.changes)
                    .and_then(|mut changes| changes.remove(&document_uri))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .flatten()
        .collect();
    if !edits.is_empty() {
        let request_id = server.bump_request_id();
        if let Err(err) = server.send_message(WorkspaceEditRequest::new(
            request_id,
            HashMap::from_iter([(document_uri, edits)]),
        )) {
            tracing::error!("Sending \"workspace/applyEdit\" request failed:\n{:?}", err);
        }
    }
}

pub(super) fn client_support_workspace_edits(server: &Server) -> bool {
    server
        .client_capabilities
        .as_ref()
        .is_some_and(|client_capabilities| {
            client_capabilities
                .workspace
                .as_ref()
                .and_then(|workspace_capabilities| workspace_capabilities.apply_edit)
                .is_some_and(|flag| flag)
                && client_capabilities
                    .workspace
                    .as_ref()
                    .and_then(|workspace_capabilities| {
                        workspace_capabilities.workspace_edit.as_ref()
                    })
                    .is_some_and(|capability| capability.document_changes.is_some_and(|flag| flag))
        })
}
