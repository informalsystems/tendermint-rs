//! Tendermint RPC definitions and types.
//!
//! ## Client
//!
//! This crate optionally provides access to different types of RPC client
//! functionality and different client transports based on which features you
//! select when using it.
//!
//! Two features are provided at present:
//!
//! * `http-client` - Provides [`HttpClient`] and [`HttpsClient`], which are
//!   basic RPC clients that interact with remote Tendermint nodes via
//!   **JSON-RPC over HTTP or HTTPS**. This client does not provide
//!   [`event::Event`] subscription functionality. See the [Tendermint RPC] for
//!   more details.
//! * `websocket-client` - Provides [`WebSocketClient`] and
//!   [`SecureWebSocketClient`], which provide full client functionality,
//!   including general RPC functionality as well as [`event::Event`]
//!   subscription functionality.
//!
//! ### Mock Clients
//!
//! Mock clients are included when either of the `http-client` or
//! `websocket-client` features are enabled to aid in testing. This includes
//! [`MockClient`], which implements both [`Client`] and [`SubscriptionClient`]
//! traits.
//!
//! [Tendermint RPC]: https://docs.tendermint.com/master/rpc/
//! [`/subscribe` endpoint]: https://docs.tendermint.com/master/rpc/#/Websocket/subscribe

#[cfg(any(feature = "http-client", feature = "websocket-client"))]
mod client;
#[cfg(any(feature = "http-client", feature = "websocket-client"))]
pub use client::{
    Client, MockClient, MockRequestMatcher, MockRequestMethodMatcher, Subscription,
    SubscriptionClient,
};

#[cfg(feature = "websocket-client")]
pub use client::{
    AsyncTungsteniteClient, SecureWebSocketClient, WebSocketClient, WebSocketClientDriver,
};
#[cfg(feature = "http-client")]
pub use client::{HttpClient, HttpsClient, HyperClient};

pub mod endpoint;
pub mod error;
pub mod event;
mod id;
mod method;
mod order;
pub mod query;
pub mod request;
pub mod response;
mod result;
mod utils;
mod version;

pub use self::{
    error::Error, id::Id, method::Method, order::Order, request::Request, request::SimpleRequest,
    response::Response, result::Result, version::Version,
};
