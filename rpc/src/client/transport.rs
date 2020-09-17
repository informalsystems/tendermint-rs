//! Tendermint RPC client implementations for different transports.

pub mod mock;

#[cfg(feature = "http-client")]
pub mod http;
#[cfg(feature = "websocket-client")]
pub mod websocket;

use crate::{Error, Result};
use tendermint::net;

// TODO(thane): Should we move this into a separate module?
/// Convenience method to extract the host and port associated with the given
/// address, but only if it's a TCP address (it fails otherwise).
pub fn get_tcp_host_port(address: net::Address) -> Result<(String, u16)> {
    match address {
        net::Address::Tcp { host, port, .. } => Ok((host, port)),
        other => Err(Error::invalid_params(&format!(
            "invalid RPC address: {:?}",
            other
        ))),
    }
}
