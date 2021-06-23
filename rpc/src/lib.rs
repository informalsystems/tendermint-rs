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
//! [Tendermint RPC]: https://docs.tendermint.com/master/rpc/
//! [`/subscribe` endpoint]: https://docs.tendermint.com/master/rpc/#/Websocket/subscribe

#[cfg(any(feature = "http-client", feature = "websocket-client"))]
mod client;
#[cfg(any(feature = "http-client", feature = "websocket-client"))]
pub use client::{
    Client, MockClient, MockRequestMatcher, MockRequestMethodMatcher, Subscription,
    SubscriptionClient,
};

#[cfg(feature = "http-client")]
pub use client::{HttpClient, HttpClientUrl};
#[cfg(feature = "websocket-client")]
pub use client::{WebSocketClient, WebSocketClientDriver, WebSocketClientUrl};

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
mod result;
mod rpc_url;
mod utils;
mod version;

pub use error::Error;
pub use id::Id;
pub use method::Method;
pub use order::Order;
pub use paging::{PageNumber, Paging, PerPage};
pub use request::{Request, SimpleRequest};
pub use response::Response;
pub use result::Result;
pub use rpc_url::{Scheme, Url};
pub use version::Version;
