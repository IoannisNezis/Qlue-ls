use crate::server::lsp::errors::{ErrorCode, LSPError};

#[derive(Debug)]
pub(super) enum CompletionError {
    LocalizationError(String),
    ResolveError(String),
    TemplateError(String, tera::Error),
    RequestError(String),
}

pub(super) fn to_lsp_error(completion_error: CompletionError) -> LSPError {
    match completion_error {
        CompletionError::LocalizationError(message) => {
            log::error!("Could not detect completion location\n{}", message);
            LSPError::new(
                ErrorCode::InternalError,
                &format!(
                    "Could not localize curor while handeling Completion-request:\n{}",
                    message
                ),
            )
        }
        CompletionError::ResolveError(message) => {
            log::error!("Could not resolve completions\n{}", message);
            LSPError::new(ErrorCode::InternalError, &message)
        }
        CompletionError::TemplateError(template, error) => {
            let message = format!("Could not render template \"{}\"\n{:?}", template, error);
            log::error!("{}", message);
            LSPError::new(ErrorCode::InternalError, &message)
        }
        CompletionError::RequestError(error) => {
            let message = format!("Completion query request failed\n{:?}", error);
            log::error!("{}", message);
            LSPError::new(ErrorCode::InternalError, &message)
        }
    }
}
