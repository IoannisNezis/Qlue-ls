use std::rc::Rc;

use futures::lock::Mutex;

use crate::server::{
    lsp::{
        errors::LSPError, DidChangeTextDocumentNotification, DidOpenTextDocumentNotification,
        DidSaveTextDocumentNotification,
    },
    Server,
};

pub(super) async fn handle_did_open_notification(
    server_rc: Rc<Mutex<Server>>,
    did_open_notification: DidOpenTextDocumentNotification,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
    let document = did_open_notification.get_text_document();
    server.state.add_document(document);
    Ok(())
}

pub(super) async fn handle_did_change_notification(
    server_rc: Rc<Mutex<Server>>,
    did_change_notification: DidChangeTextDocumentNotification,
) -> Result<(), LSPError> {
    let mut server = server_rc.lock().await;
    let uri = &did_change_notification.params.text_document.base.uri;
    server
        .state
        .change_document(uri, did_change_notification.params.content_changes)?;

    Ok(())
}

pub(super) async fn handle_did_save_notification(
    _server: Rc<Mutex<Server>>,
    did_save_notification: DidSaveTextDocumentNotification,
) -> Result<(), LSPError> {
    log::warn!(
        "saved text document (has no effect yet): \"{}\"",
        did_save_notification.params.text_document.uri
    );
    Ok(())
}
