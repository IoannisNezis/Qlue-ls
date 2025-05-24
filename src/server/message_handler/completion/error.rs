use crate::server::lsp::errors::{ErrorCode, LSPError};

#[derive(Debug)]
pub(super) enum CompletionError {
    Localization(String),
    Resolve(String),
    Template(String, tera::Error),
    Request(String),
}

pub(super) fn to_lsp_error(completion_error: CompletionError) -> LSPError {
    match completion_error {
        CompletionError::Localization(message) => {
            log::error!("Could not detect completion location\n{}", message);
            LSPError::new(
                ErrorCode::InternalError,
                &format!(
                    "Could not localize curor while handeling Completion-request:\n{}",
                    message
                ),
            )
        }
        CompletionError::Resolve(message) => {
            log::error!("Could not resolve completions\n{}", message);
            LSPError::new(ErrorCode::InternalError, &message)
        }
        CompletionError::Template(template, error) => {
            let message = format!("Could not render template \"{}\"\n{:?}", template, error);
            log::error!("{}", message);
            LSPError::new(ErrorCode::InternalError, &message)
        }
        CompletionError::Request(error) => {
            let message = format!("Completion query request failed\n{:?}", error);
            log::error!("{}", message);
            LSPError::new(ErrorCode::InternalError, &message)
        }
    }
}
