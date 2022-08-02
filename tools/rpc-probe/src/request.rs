//! Messages for interacting with the Tendermint RPC.

use std::fmt;

use serde_json::json;

use crate::utils::uuid_v4;

#[derive(Debug, Clone)]
pub struct Request {
    /// The JSON-RPC request method.
    pub method: String,
    /// The parameters for the specific method.
    pub params: serde_json::Value,
    // The ID of this request (auto-generated).
    id: String,
}

impl Request {
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        Self {
            method: method.to_owned(),
            params,
            id: uuid_v4(),
        }
    }

    pub fn as_json(&self) -> serde_json::Value {
        json!({
            "jsonrpc": "2.0",
            "id": self.id,
            "method": self.method,
            "params": self.params,
        })
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let wrapper = self.as_json();
        write!(f, "{}", serde_json::to_string_pretty(&wrapper).unwrap())
    }
}
