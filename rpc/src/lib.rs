//! Tendermint RPC definitions and types.
//!
//! ## Client
//!
//! This crate optionally provides access to different types of RPC client
//! functionality and different client transports based on which features you
//! select when using it.
//!
//! Several client-related features are provided at present:
//!
//! * `http-client` - Provides [`HttpClient`], which is a basic RPC client that interacts with
//!   remote Tendermint nodes via **JSON-RPC over HTTP or HTTPS**. This client does not provide
//!   [`event::Event`] subscription functionality. See the [Tendermint RPC] for more details.
//! * `websocket-client` - Provides [`WebSocketClient`], which provides full client functionality,
//!   including general RPC functionality as well as [`event::Event`] subscription functionality.
//!   Can be used over secure (`wss://`) and unsecure (`ws://`) connections.
//!
//! ### Mock Clients
//!
//! Mock clients are included when either of the `http-client` or
//! `websocket-client` features are enabled to aid in testing. This includes
//! [`MockClient`], which implements both [`Client`] and [`SubscriptionClient`]
//! traits.
//!
//! [Tendermint RPC]: https://docs.tendermint.com/v0.34/rpc/
//! [`/subscribe` endpoint]: https://docs.tendermint.com/v0.34/rpc/#/Websocket/subscribe

#![no_std]

extern crate alloc;
extern crate std;

mod prelude;

#[cfg(any(feature = "http-client", feature = "websocket-client"))]
mod client;
#[cfg(feature = "http-client")]
pub use client::{HttpClient, HttpClientUrl};
#[cfg(any(feature = "http-client", feature = "websocket-client"))]
pub use client::{
    MockClient, MockRequestMatcher, MockRequestMethodMatcher, Subscription, SubscriptionClient,
};
#[cfg(feature = "websocket-client")]
pub use client::{WebSocketClientUrl, WebSocketConfig};

pub mod dialect;
pub mod endpoint;
pub mod error;
pub mod event;
mod id;
mod method;
mod order;
mod paging;
pub mod query;
pub mod request;
pub mod response;
pub mod response_error;
mod rpc_url;
pub mod serializers;
mod utils;
mod version;

pub mod v0_34;
pub mod v0_37;

#[cfg(any(feature = "http-client", feature = "websocket-client"))]
pub use v0_37::Client;
#[cfg(feature = "websocket-client")]
pub use v0_37::{WebSocketClient, WebSocketClientDriver};

pub use error::Error;
pub use id::Id;
pub use method::Method;
pub use order::Order;
pub use paging::{PageNumber, Paging, PerPage};
pub use request::{Request, SimpleRequest};
pub use response::Response;
pub use response_error::{Code, ResponseError};
pub use rpc_url::{Scheme, Url};
pub use version::Version;
