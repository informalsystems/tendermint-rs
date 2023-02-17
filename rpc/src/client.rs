//! Tendermint RPC client.

mod compat;
pub use compat::CompatMode;
mod subscription;
pub use subscription::{Subscription, SubscriptionClient};
pub mod sync;

mod transport;

#[cfg(feature = "http-client")]
pub use transport::http::{HttpClient, HttpClientUrl, HttpConfig};
pub use transport::mock::{MockClient, MockRequestMatcher, MockRequestMethodMatcher};
#[cfg(feature = "websocket-client")]
pub use transport::websocket::{
    WebSocketClient, WebSocketClientDriver, WebSocketClientUrl, WebSocketConfig,
};
