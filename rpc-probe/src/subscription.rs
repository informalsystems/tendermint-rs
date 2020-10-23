//! Subscription-related functionality for the WebSocket-based client.

use crate::error::Result;
use tokio::sync::mpsc;

/// We receive events (serialized JSON-RPC responses) via a subscription.
pub type SubscriptionRx = mpsc::UnboundedReceiver<Result<serde_json::Value>>;

/// The sending end of a subscription channel.
pub type SubscriptionTx = mpsc::UnboundedSender<Result<serde_json::Value>>;
