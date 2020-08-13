//! Tendermint RPC definitons and types.

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub use client::{event_listener, new_subscription_client, transport, Client, SubscriptionClient};

pub mod endpoint;
pub mod error;
pub mod event;
mod id;
mod method;
pub mod request;
pub mod response;
mod version;

pub use self::{
    error::Error, id::Id, method::Method, request::Request, response::Response, version::Version,
};
