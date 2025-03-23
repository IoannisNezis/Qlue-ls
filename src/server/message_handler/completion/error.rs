use crate::server::lsp::errors::{ErrorCode, LSPError};

#[derive(Debug)]
pub(super) enum CompletionError {
    LocalizationError(LocalizationError),
}

impl CompletionError {
    pub(super) fn localization_error(code: ErrorCode, message: String) -> Self {
        CompletionError::LocalizationError(LocalizationError { code, message })
    }
}

pub(super) fn to_resonse_error(completion_error: CompletionError) -> LSPError {
    match completion_error {
        CompletionError::LocalizationError(localization_error) => LSPError::new(
            localization_error.code,
            &format!(
                "Could not localize curor while handeling Completion-request:\n{}",
                localization_error.message
            ),
        ),
    }
}

#[derive(Debug)]
pub(super) struct LocalizationError {
    code: ErrorCode,
    message: String,
}
