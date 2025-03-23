mod backend;
mod code_action;
mod completion;
mod diagnostic;
mod formatting;
mod hovering;
mod identification;
mod lifecycle;
mod misc;
mod textdocument_syncronization;

use backend::handle_set_backend_request;
use code_action::handle_codeaction_request;
use completion::handle_completion_request;
use diagnostic::handle_diagnostic_request;
use hovering::handle_hover_request;
use lifecycle::{
    handle_exit_notifcation, handle_initialize_request, handle_initialized_notifcation,
    handle_shutdown_request,
};
use log::warn;
use misc::handle_set_trace_notifcation;
use textdocument_syncronization::{
    handle_did_change_notification, handle_did_open_notification, handle_did_save_notification,
};

pub use formatting::format_raw;

use crate::server::lsp::errors::ErrorCode;

use self::formatting::handle_format_request;

use super::{
    lsp::{errors::LSPError, rpc::deserialize_message},
    Server,
};

pub(super) async fn dispatch(server: &mut Server, message_string: &String) -> Result<(), LSPError> {
    let message = deserialize_message(message_string)?;
    let method = message.get_method().unwrap_or("");
    macro_rules! link {
        ($handler:ident) => {
            $handler(server, message.parse()?).await
        };
    }
    match method {
        // NOTE: Requests
        "initialize" => link!(handle_initialize_request),
        "shutdown" => link!(handle_shutdown_request),
        "textDocument/formatting" => link!(handle_format_request),
        "textDocument/diagnostic" => link!(handle_diagnostic_request),
        "textDocument/codeAction" => link!(handle_codeaction_request),
        "textDocument/hover" => link!(handle_hover_request),
        "textDocument/completion" => link!(handle_completion_request),
        // NOTE: Notifications
        "initialized" => link!(handle_initialized_notifcation),
        "exit" => link!(handle_exit_notifcation),
        "textDocument/didOpen" => link!(handle_did_open_notification),
        "textDocument/didChange" => link!(handle_did_change_notification),
        "textDocument/didSave" => link!(handle_did_save_notification),
        "$/setTrace" => link!(handle_set_trace_notifcation),
        // NOTE: LSP extensions
        // Requests
        "qlueLs/setBackend" => link!(handle_set_backend_request),
        unknown_method => {
            warn!(
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
