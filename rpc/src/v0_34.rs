#[cfg(any(feature = "http-client", feature = "websocket-client"))]
mod client;
#[cfg(any(feature = "http-client", feature = "websocket-client"))]
pub use client::Client;

pub mod dialect;
pub use dialect::Dialect;
