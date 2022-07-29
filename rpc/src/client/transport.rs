//! Tendermint RPC client implementations for different transports.

mod auth;
pub mod mock;
mod router;

#[cfg(feature = "http-client")]
pub mod http;
#[cfg(feature = "websocket-client")]
pub mod websocket;
