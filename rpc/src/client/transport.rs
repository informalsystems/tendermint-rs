//! Tendermint RPC client implementations for different transports.

mod auth;
pub mod mock;
mod router;

macro_rules! perform_with_compat {
    ($self:expr, $request:expr) => {{
        let request = $request;
        match $self.compat {
            CompatMode::V0_37 => $self.perform(request).await,
            CompatMode::V0_34 => $self.perform_v0_34(request).await,
        }
    }};
}

#[cfg(feature = "http-client")]
pub mod http;
#[cfg(feature = "http-client")]
mod proxy;
#[cfg(feature = "websocket-client")]
pub mod websocket;
