//! Client transport-related utilities.

use crate::{Error, Result};
use tendermint::net;

/// Convenience method to extract the host and port associated with the given
/// address, but only if it's a TCP address (it fails otherwise).
pub(crate) fn get_tcp_host_port(address: net::Address) -> Result<(String, u16)> {
    match address {
        net::Address::Tcp { host, port, .. } => Ok((host, port)),
        other => Err(Error::invalid_params(&format!(
            "invalid RPC address: {:?}",
            other
        ))),
    }
}
