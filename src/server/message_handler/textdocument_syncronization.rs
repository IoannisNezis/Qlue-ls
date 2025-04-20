use std::{cell::RefCell, rc::Rc};

use log::{error, info, warn};

use crate::server::{
    lsp::{
        errors::{ErrorCode, LSPError},
        DidChangeTextDocumentNotification, DidOpenTextDocumentNotification,
        DidSaveTextDocumentNotification,
    },
    Server,
};

pub(super) async fn handle_did_open_notification(
    server_rc: Rc<RefCell<Server>>,
    did_open_notification: DidOpenTextDocumentNotification,
) -> Result<(), LSPError> {
    let mut server = server_rc.borrow_mut();
    info!(
        "opened text document: \"{}\"",
        did_open_notification.params.text_document.uri
    );
    let document = did_open_notification.get_text_document();
    let tree = server.tools.parser.parse(document.text.as_bytes(), None);
    server.state.add_document(document, tree);
    Ok(())
}

pub(super) async fn handle_did_change_notification(
    server_rc: Rc<RefCell<Server>>,
    did_change_notification: DidChangeTextDocumentNotification,
) -> Result<(), LSPError> {
    let uri = &did_change_notification.params.text_document.base.uri;
    server_rc
        .borrow_mut()
        .state
        .change_document(uri, did_change_notification.params.content_changes)?;
    let text = server_rc.borrow().state.get_document(uri)?.text.clone();
    // let old_tree = server.state.get_tree(&uri).ok();
    let new_tree = server_rc
        .borrow_mut()
        .tools
        .parser
        .parse(text.as_bytes(), None);
    if new_tree.is_none() {
        warn!("Could not build new parse-tree for \"{}\"", uri);
    }
    if let Err(err) = server_rc.borrow_mut().state.update_tree(uri, new_tree) {
        error!("{}", err.message);
        return Err(LSPError::new(
            ErrorCode::InternalError,
            &format!("Error while building parse-tree:\n{}", err.message),
        ));
    }

    Ok(())
}

pub(super) async fn handle_did_save_notification(
    _server: Rc<RefCell<Server>>,
    did_save_notification: DidSaveTextDocumentNotification,
) -> Result<(), LSPError> {
    log::warn!(
        "saved text document (has no effect yet): \"{}\"",
        did_save_notification.params.text_document.uri
    );
    Ok(())
}
