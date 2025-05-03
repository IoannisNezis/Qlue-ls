mod backend;
mod code_action;
mod completion;
mod diagnostic;
mod formatting;
mod hover;
mod identification;
mod jump;
mod lifecycle;
mod misc;
mod textdocument_syncronization;

use std::rc::Rc;

use backend::{
    handle_add_backend_notification, handle_ping_backend_request,
    handle_update_backend_default_notification,
};
use code_action::handle_codeaction_request;
use completion::handle_completion_request;
use diagnostic::handle_diagnostic_request;
use futures::lock::Mutex;
use hover::handle_hover_request;
use jump::handle_jump_request;
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

use crate::server::{handle_error, lsp::errors::ErrorCode};

use self::formatting::handle_format_request;

use super::{
    lsp::{errors::LSPError, rpc::deserialize_message},
    Server,
};

pub(super) async fn dispatch(
    server_rc: Rc<Mutex<Server>>,
    message_string: &String,
) -> Result<(), LSPError> {
    let message = deserialize_message(message_string)?;
    let method = message.get_method().unwrap_or("");
    macro_rules! call {
        ($handler:ident) => {
            $handler(server_rc, message.parse()?).await
        };
    }
    match method {
        // NOTE: Requests
        "initialize" => call!(handle_initialize_request),
        "shutdown" => call!(handle_shutdown_request),
        "textDocument/formatting" => call!(handle_format_request),
        "textDocument/diagnostic" => call!(handle_diagnostic_request),
        "textDocument/codeAction" => call!(handle_codeaction_request),
        "textDocument/hover" => call!(handle_hover_request),
        "textDocument/completion" => {
            let message_copy = message_string.clone();
            spawn_local(async move {
                if let Err(err) =
                    handle_completion_request(server_rc.clone(), message.parse().unwrap()).await
                {
                    handle_error(server_rc, &message_copy, err).await;
                }
            });
            Ok(())
        }
        // NOTE: Notifications
        "initialized" => call!(handle_initialized_notifcation),
        "exit" => call!(handle_exit_notifcation),
        "textDocument/didOpen" => call!(handle_did_open_notification),
        "textDocument/didChange" => {
            call!(handle_did_change_notification)
        }
        "textDocument/didSave" => call!(handle_did_save_notification),
        "$/setTrace" => call!(handle_set_trace_notifcation),
        // NOTE: LSP extensions
        // Requests
        "qlueLs/addBackend" => call!(handle_add_backend_notification),
        "qlueLs/updateDefaultBackend" => call!(handle_update_backend_default_notification),
        "qlueLs/pingBackend" => {
            let message_copy = message_string.clone();
            spawn_local(async move {
                if let Err(err) =
                    handle_ping_backend_request(server_rc.clone(), message.parse().unwrap()).await
                {
                    handle_error(server_rc, &message_copy, err).await;
                }
            });
            Ok(())
        }
        "qlueLs/jump" => {
            call!(handle_jump_request)
        }
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
