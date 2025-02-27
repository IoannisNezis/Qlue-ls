use crate::server::lsp::errors::{ErrorCode, ResponseError};

pub(super) enum CompletionError {
    LocalizationError(LocalizationError),
}

impl CompletionError {
    pub(super) fn localization_error(code: ErrorCode, message: String) -> Self {
        CompletionError::LocalizationError(LocalizationError { code, message })
    }
}

pub(super) fn to_resonse_error(completion_error: CompletionError) -> ResponseError {
    match completion_error {
        CompletionError::LocalizationError(localization_error) => ResponseError::new(
            localization_error.code,
            &format!(
                "Could not localize curor while handeling Completion-request:\n{}",
                localization_error.message
            ),
        ),
    }
}

struct LocalizationError {
    code: ErrorCode,
    message: String,
}
