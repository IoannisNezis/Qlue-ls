use std::{collections::HashMap, rc::Rc};

use futures::lock::Mutex;
use ll_sparql_parser::{ast::AstNode, ast::Var, syntax_kind::SyntaxKind};
use crate::{
    Server,
    server::message_handler::common::{find_variable_occurrences, is_valid_variable_name},
    server::lsp::{
        RenameRequest, RenameResponse, WorkspaceEdit,
        errors::{ErrorCode, LSPError},
        textdocument::{Range, TextEdit},
    },
};

pub(super) async fn handle_rename_request(
    server_rc: Rc<Mutex<Server>>,
    request: RenameRequest,
) -> Result<(), LSPError> {
    let (root, document_text) = {
        let server = server_rc.lock().await;
        let document = server.state.get_document(request.get_document_uri())?;
        (
            server
                .state
                .get_cached_parse_tree(request.get_document_uri())?
                .tree,
            document.text.clone(),
        )
    };
    let offset = request
        .get_position()
        .byte_index(&document_text)
        .ok_or_else(|| {
            LSPError::new(
                ErrorCode::InvalidParams,
                "The hover position is not inside the text document",
            )
        })?;
    let token = root.token_at_offset(offset).left_biased().ok_or_else(|| {
        LSPError::new(
            ErrorCode::InvalidParams,
            "The parse tree does not have a token at this offset.",
        )
    })?;
    if !matches!(token.kind(), SyntaxKind::VAR1 | SyntaxKind::VAR2) {
        return Err(LSPError::new(
            ErrorCode::RequestFailed,
            "Renaming is only provided for variables.",
        ));
    }

    // NOTE: tolerate a new name given with a leading `?` or `$`
    let raw_new_name = request.get_new_name();
    let new_name = raw_new_name
        .strip_prefix(['?', '$'])
        .unwrap_or(raw_new_name);
    if !is_valid_variable_name(&new_name) {
        return Err(LSPError::new(
            ErrorCode::InvalidParams,
            &format!("\"{new_name}\" is not a valid variable name."),
        ));
    }

    let variable = token
        .parent()
        .and_then(Var::cast)
        .ok_or_else(|| LSPError::new(ErrorCode::InternalError, "Variable token has no Var node"))?;
    let variables = find_variable_occurrences(&variable);

    let workspace_edit = WorkspaceEdit {
        changes: Some(HashMap::from_iter([(
            request.get_document_uri().clone(),
            variables
                .into_iter()
                // NOTE: edits always use the `?` sigil, so occurrences written
                // with `$` are intentionally normalized to `?`
                .map(|var| TextEdit {
                    range: Range::from_byte_offset_range(var.syntax().text_range(), &document_text)
                        .unwrap(),
                    new_text: format!("?{new_name}"),
                })
                .collect(),
        )])),
    };
    let mut response = RenameResponse::new(request.get_id());
    response.set_edit(workspace_edit);
    server_rc.lock().await.send_message(response)
}
