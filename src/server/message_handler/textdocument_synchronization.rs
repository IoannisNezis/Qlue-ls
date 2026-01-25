use std::rc::Rc;

use futures::lock::Mutex;
use ll_sparql_parser::parse;

use crate::server::{
    Server,
    lsp::{
        DidChangeTextDocumentNotification, DidOpenTextDocumentNotification,
        DidSaveTextDocumentNotification, PublishDiagnosticsNotification,
        diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity},
        errors::LSPError,
        textdocument::Range,
    },
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
    server_rc: Rc<Mutex<Server>>,
    did_save_notification: DidSaveTextDocumentNotification,
) -> Result<(), LSPError> {
    let uri = did_save_notification.params.text_document.uri;
    let server = server_rc.lock().await;
    let document = server.state.get_document(&uri)?;
    let (_, errors) = parse(&document.text);
    server.send_message(PublishDiagnosticsNotification::new(
        uri,
        errors
            .into_iter()
            .take(1)
            .map(|error| Diagnostic {
                range: Range::from_byte_offset_range(error.span, &document.text)
                    .expect("Parse error spans should be within the document"),
                severity: DiagnosticSeverity::Error,
                code: Some(DiagnosticCode::String("parse error".to_string())),
                source: Some("Qlue-ls".to_string()),
                message: error.message,
                data: None,
            })
            .collect(),
        None,
    ))?;
    Ok(())
}
