//! qlue-ls language server library.
//!
//! This crate provides the core language server implementation for SPARQL,
//! usable from both native and WASM targets.
//!
//! # Native Usage
//!
//! For native builds (including tests), use the re-exported server types:
//! - [`Server`] (or [`LspServer`]): The main server struct
//! - [`handle_message`] (or [`handle_lsp_message`]): Process incoming LSP messages
//! - [`format_raw`]: Format SPARQL queries directly
//! - [`format_with_settings`]: Format with custom settings
//!
//! # WASM Usage
//!
//! For WASM builds, additional functions are available:
//! - [`init_language_server`]: Creates a new server instance with a Web Streams writer
//! - [`listen`]: Main event loop using Web Streams API for I/O
//!
//! # Related Modules
//!
//! - `server`: Core server implementation shared across all targets
//! - `main.rs` (native only): CLI entry point using stdio

mod server;
mod sparql;

// Re-export core server types for all targets (used by tests and native builds)
pub use crate::server::configuration::FormatSettings;
pub use crate::server::message_handler::formatting::{format_raw, format_with_settings};
pub use crate::server::{handle_message, Server};

// Aliases for more descriptive names (for external consumers)
pub use crate::server::{handle_message as handle_lsp_message, Server as LspServer};

// WASM-specific imports and exports
#[cfg(target_family = "wasm")]
mod wasm {
    use super::server::{Server, handle_message};
    use futures::lock::Mutex;
    use log::error;
    use std::panic;
    use std::rc::Rc;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::js_sys;

    fn send_message(writer: &web_sys::WritableStreamDefaultWriter, message: String) {
        let _future = JsFuture::from(writer.write_with_chunk(&message.into()));
    }

    #[wasm_bindgen]
    pub fn init_language_server(writer: web_sys::WritableStreamDefaultWriter) -> Server {
        wasm_logger::init(wasm_logger::Config::default());
        panic::set_hook(Box::new(|info| {
            let msg = info.to_string();
            web_sys::console::error_1(&msg.into());
            let _ = js_sys::Function::new_with_args("msg", "self.postMessage({type:'crash'});")
                .call0(&JsValue::NULL);
        }));
        Server::new(move |message| send_message(&writer, message))
    }

    async fn read_message(
        reader: &web_sys::ReadableStreamDefaultReader,
    ) -> Result<(String, bool), String> {
        match JsFuture::from(reader.read()).await {
            Ok(js_object) => {
                let value = js_sys::Reflect::get(&js_object, &"value".into())
                    .map_err(|_| "\"value\" property not present in message")?
                    .as_string()
                    .ok_or("\"value\" is not a string")?;
                let done = js_sys::Reflect::get(&js_object, &"done".into())
                    .map_err(|_| "\"done\" property not present in message")?
                    .as_bool()
                    .ok_or("\"done\" is not a boolean")?;
                Ok((value, done))
            }
            Err(_) => Err("Error while reading from input-stream".to_string()),
        }
    }

    #[wasm_bindgen]
    pub async fn listen(server: Server, reader: web_sys::ReadableStreamDefaultReader) {
        let server_rc = Rc::new(Mutex::new(server));
        loop {
            match read_message(&reader).await {
                Ok((value, done)) => {
                    handle_message(server_rc.clone(), value).await;
                    if done {
                        break;
                    }
                }
                Err(e) => error!("{}", e),
            }
        }
    }
}

#[cfg(target_family = "wasm")]
pub use wasm::*;
