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

#[cfg(not(target_arch = "wasm32"))]
use tokio::task::spawn_local;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::server::{handle_error, lsp::errors::ErrorCode};

use self::formatting::handle_format_request;

use super::{
    lsp::{errors::LSPError, rpc::deserialize_message},
    Server,
};

pub(super) async fn dispatch(
    server_rc: Rc<Mutex<Server>>,
    message_string: &str,
) -> Result<(), LSPError> {
    let message = deserialize_message(message_string)?;
    let method = message.get_method().unwrap_or("");
    macro_rules! call {
        ($handler:ident) => {
            $handler(server_rc, message.parse()?).await
        };
    }

    macro_rules! call_async {
        ($handler:ident) => {{
            let message_copy = message_string.to_string();
            let task = spawn_local(async move {
                if let Err(err) = $handler(server_rc.clone(), message.parse().unwrap()).await {
                    handle_error(server_rc, &message_copy, err).await;
                }
            });
            #[cfg(not(target_arch = "wasm32"))]
            task.await.expect("local task should not crash");
            Ok(())
        }};
    }

    match method {
        // NOTE: Requests
        "initialize" => call!(handle_initialize_request),
        "shutdown" => call!(handle_shutdown_request),
        "textDocument/formatting" => call!(handle_format_request),
        "textDocument/diagnostic" => call!(handle_diagnostic_request),
        "textDocument/codeAction" => call!(handle_codeaction_request),
        "textDocument/hover" => call_async!(handle_hover_request),
        "textDocument/completion" => call_async!(handle_completion_request),
        // NOTE: Notifications
        "initialized" => call!(handle_initialized_notifcation),
        "exit" => call!(handle_exit_notifcation),
        "textDocument/didOpen" => call!(handle_did_open_notification),
        "textDocument/didChange" => call!(handle_did_change_notification),
        "textDocument/didSave" => call!(handle_did_save_notification),
        "$/setTrace" => call!(handle_set_trace_notifcation),
        // NOTE: LSP extensions Requests
        "qlueLs/addBackend" => call!(handle_add_backend_notification),
        "qlueLs/updateDefaultBackend" => call!(handle_update_backend_default_notification),
        "qlueLs/pingBackend" => call_async!(handle_ping_backend_request),
        "qlueLs/jump" => call!(handle_jump_request),
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
