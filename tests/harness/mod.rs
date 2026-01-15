//! Test harness for end-to-end LSP testing
//!
//! This module provides a `TestClient` that wraps the LSP server and captures
//! all outgoing messages for verification in tests.

pub mod helpers;
pub mod runtime;

use std::cell::RefCell;
use std::rc::Rc;

use futures::lock::Mutex;
use qlue_ls::{LspServer, handle_lsp_message};
use serde_json::Value;

/// Collects all messages sent by the server
pub struct CapturedMessages {
    messages: RefCell<Vec<String>>,
}

impl CapturedMessages {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            messages: RefCell::new(Vec::new()),
        })
    }

    pub fn push(&self, message: String) {
        self.messages.borrow_mut().push(message);
    }

    pub fn take_all(&self) -> Vec<String> {
        std::mem::take(&mut *self.messages.borrow_mut())
    }

    pub fn get_all(&self) -> Vec<String> {
        self.messages.borrow().clone()
    }

    pub fn last(&self) -> Option<String> {
        self.messages.borrow().last().cloned()
    }

    pub fn len(&self) -> usize {
        self.messages.borrow().len()
    }

    /// Find a response message by its request ID
    pub fn find_by_id(&self, id: u32) -> Option<String> {
        self.messages
            .borrow()
            .iter()
            .find(|msg| {
                serde_json::from_str::<Value>(msg)
                    .ok()
                    .and_then(|v| v.get("id")?.as_u64())
                    .map(|msg_id| msg_id == id as u64)
                    .unwrap_or(false)
            })
            .cloned()
    }

    /// Find all notification messages (messages without an id that are responses)
    pub fn find_notifications(&self, method: &str) -> Vec<Value> {
        self.messages
            .borrow()
            .iter()
            .filter_map(|msg| {
                let v: Value = serde_json::from_str(msg).ok()?;
                if v.get("method")?.as_str()? == method {
                    Some(v)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for CapturedMessages {
    fn default() -> Self {
        Self {
            messages: RefCell::new(Vec::new()),
        }
    }
}

/// Test client for end-to-end LSP testing
pub struct TestClient {
    server: Rc<Mutex<LspServer>>,
    pub captured: Rc<CapturedMessages>,
    request_id_counter: RefCell<u32>,
}

impl TestClient {
    /// Create a new test client with a fresh server instance
    pub fn new() -> Self {
        let captured = CapturedMessages::new();
        let captured_clone = captured.clone();

        let server = LspServer::new(move |message| {
            captured_clone.push(message);
        });

        Self {
            server: Rc::new(Mutex::new(server)),
            captured,
            request_id_counter: RefCell::new(1),
        }
    }

    /// Get the next request ID
    fn next_id(&self) -> u32 {
        let id = *self.request_id_counter.borrow();
        *self.request_id_counter.borrow_mut() += 1;
        id
    }

    /// Send a raw JSON-RPC message string to the server
    pub async fn send_raw(&self, message: &str) {
        handle_lsp_message(self.server.clone(), message.to_string()).await;
    }

    /// Send a request and return the request ID
    pub async fn send_request(&self, method: &str, params: Value) -> u32 {
        let id = self.next_id();
        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        self.send_raw(&message.to_string()).await;
        id
    }

    /// Send a notification (no response expected)
    pub async fn send_notification(&self, method: &str, params: Value) {
        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        self.send_raw(&message.to_string()).await;
    }

    /// Get the response for a specific request ID
    pub fn get_response(&self, id: u32) -> Option<Value> {
        self.captured
            .find_by_id(id)
            .and_then(|msg| serde_json::from_str(&msg).ok())
    }

    /// Get the last captured message as JSON
    pub fn last_message(&self) -> Option<Value> {
        self.captured
            .last()
            .and_then(|msg| serde_json::from_str(&msg).ok())
    }

    /// Get all captured messages as JSON
    pub fn all_messages(&self) -> Vec<Value> {
        self.captured
            .get_all()
            .iter()
            .filter_map(|msg| serde_json::from_str(msg).ok())
            .collect()
    }

    /// Clear all captured messages
    pub fn clear_messages(&self) {
        self.captured.take_all();
    }
}

impl Default for TestClient {
    fn default() -> Self {
        Self::new()
    }
}
