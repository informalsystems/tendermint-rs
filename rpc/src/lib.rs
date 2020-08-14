//! Tendermint RPC definitions and types.

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::{
    FullClient, MinimalClient, Subscription, SubscriptionId, SubscriptionRouter,
    DEFAULT_SUBSCRIPTION_BUF_SIZE,
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
pub mod result;
mod version;

pub use self::{
    error::Error, id::Id, method::Method, request::Request, response::Response, result::Result,
    version::Version,
};
