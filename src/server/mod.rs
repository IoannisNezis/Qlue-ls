mod anaysis;
mod capabilities;
mod common;
mod configuration;
mod fetch;
mod lsp;
mod state;
mod tools;

mod message_handler;

use std::{any::type_name, rc::Rc};

use capabilities::create_capabilities;
use configuration::Settings;
use futures::lock::Mutex;
use log::{error, info};
use lsp::{
    errors::{ErrorCode, LSPError},
    rpc::{RecoverId, RequestIdOrNull, ResponseMessage},
    ServerInfo,
};
use message_handler::dispatch;

// WARNING: This is a temporary soloution to export the format function directly
// will remove soon (12.12.24)
#[allow(unused_imports)]
pub use message_handler::format_raw;

use serde::Serialize;
use state::ServerState;
use tools::Tools;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct Server {
    pub(crate) state: ServerState,
    pub(crate) settings: Settings,
    pub(crate) capabilities: lsp::capabilities::ServerCapabilities,
    pub(crate) server_info: ServerInfo,
    tools: Tools,
    send_message_clusure: Box<dyn Fn(String)>,
}

impl Server {
    pub fn new(write_function: impl Fn(String) + 'static) -> Server {
        let version = env!("CARGO_PKG_VERSION");
        info!("Started Language Server: Qlue-ls - version: {}", version);
        Self {
            state: ServerState::new(),
            settings: Settings::new(),
            capabilities: create_capabilities(),
            server_info: ServerInfo {
                name: "Qlue-ls".to_string(),
                version: Some(version.to_string()),
            },
            tools: Tools::init(),
            send_message_clusure: Box::new(write_function),
        }
    }

    pub fn get_version(&self) -> String {
        self.server_info
            .version
            .clone()
            .unwrap_or("not-specified".to_string())
    }

    fn send_message<T>(&self, message: T) -> Result<(), LSPError>
    where
        T: Serialize,
    {
        let message_string = serde_json::to_string(&message).map_err(|error| {
            LSPError::new(
                ErrorCode::ParseError,
                &format!(
                    "Could not deserialize RPC-message \"{}\"\n\n{}",
                    type_name::<T>(),
                    error
                ),
            )
        })?;
        (self.send_message_clusure)(message_string);
        Ok(())
    }

    /// Shortens a raw URI into its CURIE (Compact URI) form and retrieves related metadata.
    ///
    /// This method takes a raw URI as input, attempts to find its associated prefix and URI prefix
    /// from the `uri_converter`, and shorten the URI into its CURIE form. If successful, it
    /// returns a tuple containing:
    /// - The prefix associated with the URI.
    /// - The URI prefix corresponding to the namespace of the URI.
    /// - The CURIE representation of the URI.
    ///
    /// # Parameters
    /// - `uri`: A string slice representing the raw URI to be shortened.
    ///
    /// # Returns
    /// - `Some((prefix, uri_prefix, curie))` if the URI can be successfully compacted:
    ///   - `prefix`: A `String` representing the prefix associated with the URI.
    ///   - `uri_prefix`: A `String` representing the URI namespace prefix.
    ///   - `curie`: A `String` representing the compact CURIE form of the URI.
    /// - `None` if the URI cannot be found or shortened.
    ///
    /// # Example
    /// ```rust
    /// let uri = "http://example.com/resource";
    /// if let Some((prefix, uri_prefix, curie)) = instance.shorten_uri(uri) {
    ///     println!("Prefix: {}", prefix);
    ///     println!("URI Prefix: {}", uri_prefix);
    ///     println!("CURIE: {}", curie);
    /// } else {
    ///     println!("Failed to shorten URI.");
    /// }
    /// ```
    ///
    /// # Errors
    /// Returns `None` if:
    /// - The `uri_converter` fails to find a record associated with the URI.
    /// - The `uri_converter` fails to shorten the URI into a CURIE.
    pub(crate) fn shorten_uri(
        &self,
        uri: &str,
        backend_name: Option<&String>,
    ) -> Option<(String, String, String)> {
        let converter = backend_name
            .and_then(|name| self.state.get_converter(name))
            .or(self.state.get_default_converter())?;
        let record = converter.find_by_uri(uri).ok()?;
        let curie = converter.compress(uri).ok()?;
        Some((record.prefix.clone(), record.uri_prefix.clone(), curie))
    }
}

async fn handle_error(server_rc: Rc<Mutex<Server>>, message: &String, error: LSPError) {
    log::error!(
        "Error occured while handling message:\n\"{}\"\n\n{:?}\n{}",
        message,
        error.code,
        error.message
    );
    if let Ok(id) =
        serde_json::from_str::<RecoverId>(&message).map(|msg| RequestIdOrNull::RequestId(msg.id))
    {
        if let Err(error) = server_rc
            .lock()
            .await
            .send_message(ResponseMessage::error(id, error))
        {
            error!(
                "CRITICAL: could not serialize error message (this very bad):\n{:?}",
                error
            )
        }
    }
}

pub async fn handle_message(server_rc: Rc<Mutex<Server>>, message: String) {
    if let Err(err) = dispatch(server_rc.clone(), &message).await {
        handle_error(server_rc.clone(), &message, err).await;
    }
}
