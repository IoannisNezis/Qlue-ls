mod backend;
mod code_action;
mod completion;
mod diagnostic;
mod formatting;
mod hover;
mod identification;
mod lifecycle;
mod misc;
mod textdocument_syncronization;

use std::{cell::RefCell, rc::Rc, sync::Mutex};

use backend::{
    handle_add_backend_notification, handle_ping_backend_request,
    handle_update_backend_default_notification,
};
use code_action::handle_codeaction_request;
use completion::handle_completion_request;
use diagnostic::handle_diagnostic_request;
use hover::handle_hover_request;
use lifecycle::{
    handle_exit_notifcation, handle_initialize_request, handle_initialized_notifcation,
    handle_shutdown_request,
};
use misc::handle_set_trace_notifcation;
use textdocument_syncronization::{
    handle_did_change_notification, handle_did_open_notification, handle_did_save_notification,
};

pub use formatting::format_raw;
use wasm_bindgen_futures::spawn_local;

use crate::server::lsp::errors::ErrorCode;

use self::formatting::handle_format_request;

use super::{
    lsp::{errors::LSPError, rpc::deserialize_message},
    Server,
};

pub(super) async fn dispatch(
    server_rc: Rc<RefCell<Server>>,
    message_string: &String,
) -> Result<(), LSPError> {
    let message = deserialize_message(message_string)?;
    let method = message.get_method().unwrap_or("");
    macro_rules! link {
        ($handler:ident) => {
            $handler(server_rc, message.parse()?).await
        };
    }
    log::info!("method: {}", method);
    match method {
        // NOTE: Requests
        "initialize" => link!(handle_initialize_request),
        "shutdown" => link!(handle_shutdown_request),
        "textDocument/formatting" => link!(handle_format_request),
        "textDocument/diagnostic" => link!(handle_diagnostic_request),
        "textDocument/codeAction" => link!(handle_codeaction_request),
        "textDocument/hover" => link!(handle_hover_request),
        "textDocument/completion" => {
            spawn_local(async move {
                handle_completion_request(server_rc, message.parse().unwrap()).await;
            });
            Ok(())
        }
        // NOTE: Notifications
        "initialized" => link!(handle_initialized_notifcation),
        "exit" => link!(handle_exit_notifcation),
        "textDocument/didOpen" => link!(handle_did_open_notification),
        "textDocument/didChange" => link!(handle_did_change_notification),
        "textDocument/didSave" => link!(handle_did_save_notification),
        "$/setTrace" => link!(handle_set_trace_notifcation),
        // NOTE: LSP extensions
        // Requests
        "qlueLs/addBackend" => link!(handle_add_backend_notification),
        "qlueLs/updateDefaultBackend" => link!(handle_update_backend_default_notification),
        "qlueLs/pingBackend" => link!(handle_ping_backend_request),
        // NOTE: Known unsupported message
        "$/cancelRequest" => {
            log::warn!("Received cancel request (unsupported)");
            Ok(())
        }
        unknown_method => {
            log::warn!(
                "Received message with unknown method \"{}\"",
                unknown_method
            );
            Err(LSPError::new(
                ErrorCode::MethodNotFound,
                &format!("Method \"{}\" currently not supported", unknown_method),
            ))
        }
    }
}
