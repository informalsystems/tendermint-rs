//! Tendermint RPC client implementations for different transports.

pub mod mock;
mod router;

#[cfg(feature = "http-client")]
pub mod http;
#[cfg(feature = "websocket-client")]
pub mod websocket;
