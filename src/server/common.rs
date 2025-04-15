use std::{any::type_name, fmt};

use log::error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::lsp::errors::{ErrorCode, LSPError};

pub(crate) fn serde_parse<T, O>(message: O) -> Result<T, LSPError>
where
    T: Serialize + DeserializeOwned,
    O: Serialize + fmt::Debug,
{
    match serde_json::to_string(&message) {
        Ok(serialized_message) => serde_json::from_str(&serialized_message).map_err(|error| {
            error!(
                "Error while deserializing message:\n{}-----------------------\n{:?}",
                error, message,
            );
            LSPError::new(
                ErrorCode::ParseError,
                &format!(
                    "Could not deserialize RPC-message \"{}\"\n\n{}",
                    type_name::<T>(),
                    error
                ),
            )
        }),
        Err(error) => Err(LSPError::new(
            ErrorCode::ParseError,
            &format!("Could not serialize RPC-message\n\n{}", error),
        )),
    }
}

/// This struct represents diagnostic data from the uncompacted-uri diagnostic.
///
/// The fields are:
/// - `prefix`: The prefix associated with the namespace.
/// - `namespace`: The namespace URI.
/// - `curie`: The compact URI (CURIE).
#[derive(Debug, Serialize, Deserialize)]
pub struct UncompactedUrisDiagnosticData(pub String, pub String, pub String);
