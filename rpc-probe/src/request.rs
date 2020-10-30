//! Messages for interacting with the Tendermint RPC.

use crate::error::Result;
use crate::utils::uuid_v4;
use crate::websocket::WebSocketClient;
use serde_json::json;
use std::fmt;

#[derive(Debug)]
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

    pub fn wrapper(&self) -> serde_json::Value {
        json!({
            "jsonrpc": "2.0",
            "id": self.id,
            "method": self.method,
            "params": self.params,
        })
    }

    pub async fn execute(&self, client: &mut WebSocketClient) -> Result<serde_json::Value> {
        client.request(self.wrapper()).await
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let wrapper = self.wrapper();
        write!(f, "{}", serde_json::to_string_pretty(&wrapper).unwrap())
    }
}
