//! Subscription-related functionality for the WebSocket-based client.

use std::fmt;

use serde_json::json;
use tokio::sync::mpsc;

use crate::{error::Result, utils::uuid_v4};

/// We receive events (serialized JSON-RPC responses) via a subscription.
pub type SubscriptionRx = mpsc::UnboundedReceiver<Result<serde_json::Value>>;

/// The sending end of a subscription channel.
pub type SubscriptionTx = mpsc::UnboundedSender<Result<serde_json::Value>>;

#[derive(Debug, Clone)]
pub struct Subscription {
    pub query: String,
    id: String,
}

impl Subscription {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_owned(),
            id: uuid_v4(),
        }
    }

    pub fn as_json(&self) -> serde_json::Value {
        json!({
            "jsonrpc": "2.0",
            "id": self.id,
            "method": "subscribe",
            "params": {
                "query": self.query,
            },
        })
    }
}

impl From<&str> for Subscription {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl fmt::Display for Subscription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let wrapper = self.as_json();
        write!(f, "{}", serde_json::to_string_pretty(&wrapper).unwrap())
    }
}
