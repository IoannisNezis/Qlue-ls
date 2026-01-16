//! LSP protocol types and JSON-RPC serialization.
//!
//! This module provides Rust types that map to the Language Server Protocol
//! specification. Types are organized by their role in the protocol.
//!
//! # Submodules
//!
//! - [`rpc`]: JSON-RPC 2.0 message framing (requests, responses, notifications)
//! - [`base_types`]: Shared types (Position, Range, Location, TextEdit)
//! - [`capabilities`]: Server and client capability structures
//! - [`textdocument`]: Document identifiers and content change events
//! - [`errors`]: LSP error codes and the `LSPError` type
//! - [`messages`]: Request/response/notification payloads (re-exported at module level)
//!
//! # Serialization
//!
//! All types derive `Serialize`/`Deserialize` for JSON conversion. Field names use
//! `camelCase` to match the LSP specification (via `#[serde(rename_all = "camelCase")]`).
//!
//! # Related Modules
//!
//! - [`super::message_handler`]: Consumes these types when handling requests
//! - [`super::Server`]: Uses `ServerCapabilities` and `ServerInfo`

pub mod base_types;
pub mod capabilities;
pub mod errors;
mod messages;
pub mod rpc;
pub mod textdocument;
mod workdoneprogress;

pub use messages::*;
