use serde::{Deserialize, Serialize};

pub mod client;
pub mod server;

/// A capability that can be either a boolean or an empty object `{}`.
///
/// Used by both client and server semantic token options for the `range` field.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum BoolOrEmpty {
    Bool(bool),
    Object {},
}

/// A capability that can be either a boolean or an object with a `delta` field.
///
/// Used by both client and server semantic token options for the `full` field.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum FullCapability {
    Bool(bool),
    Object(FullCapabilityOptions),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct FullCapabilityOptions {
    /// The server/client supports deltas for full documents.
    pub delta: Option<bool>,
}
