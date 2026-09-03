use std::{any::type_name, fmt};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tracing::error;

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

/// Milliseconds since an arbitrary epoch.
///
/// WARNING: Only differences between two calls are meaningful. The native and
/// web assembly implementations count from different epochs (Unix time and
/// `performance.now()` respectively), so the absolute value must not be
/// reported as a timestamp.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn get_timestamp_ms() -> f64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_secs_f64()
        * 1000.0
}

/// See the native implementation for the epoch caveat.
///
/// NOTE: Resolves `performance` on the worker global scope, the language server
/// always runs inside a web worker.
#[cfg(target_arch = "wasm32")]
pub(crate) fn get_timestamp_ms() -> f64 {
    use wasm_bindgen::JsCast;
    use web_sys::WorkerGlobalScope;
    let worker_global: WorkerGlobalScope = js_sys::global().unchecked_into();
    worker_global
        .performance()
        .expect("performance should be available")
        .now()
}
