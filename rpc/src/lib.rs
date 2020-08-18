//! Tendermint RPC definitions and types.
//!
//! ## Client
//!
//! Available when specifying the `client` feature flag.
//!
//! The RPC client comes in two flavors: a [`MinimalClient`] and a
//! [`FullClient`]. A `MinimalClient` implementation provides access to all
//! RPC endpoints with the exception of the [`Event`] subscription ones,
//! whereas a `FullClient` implementation provides access to all RPC
//! functionality. The reason for this distinction is because `Event`
//! subscription usually requires more resources to manage, and may not be
//! necessary for all applications making use of the Tendermint RPC.
//!
//! Transport-specific client support is provided by way of additional feature
//! flags (where right now we only have one transport, but intend on providing
//! more in future):
//!
//! * `http_ws`: Provides an HTTP interface for request/response interactions
//!   (see [`HttpClient`]), and a WebSocket-based interface for `Event`
//!   subscription (see [`HttpWebSocketClient`]).
//!
//! [`MinimalClient`]: trait.MinimalClient.html
//! [`FullClient`]: trait.FullClient.html
//! [`Event`]: event/struct.Event.html
//! [`HttpClient`]: struct.HttpClient.html
//! [`HttpWebSocketClient`]: struct.HttpWebSocketClient.html

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::{
    EventRx, EventTx, FullClient, MinimalClient, PendingResultTx, Subscription, SubscriptionId,
    SubscriptionRouter, DEFAULT_SUBSCRIPTION_BUF_SIZE,
};
#[cfg(feature = "http_ws")]
pub use client::{HttpClient, HttpWebSocketClient};

pub mod endpoint;
pub mod error;
pub mod event;
mod id;
mod method;
pub mod request;
pub mod response;
mod result;
mod version;

pub use self::{
    error::Error, id::Id, method::Method, request::Request, response::Response, result::Result,
    version::Version,
};
