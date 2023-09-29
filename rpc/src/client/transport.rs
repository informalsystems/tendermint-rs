//! Tendermint RPC client implementations for different transports.

mod auth;
pub mod mock;
mod router;

macro_rules! perform_with_compat {
    ($self:expr, $request:expr) => {{
        let request = $request;
        match $self.compat {
            CompatMode::V0_37 => {
                $self
                    .perform_with_dialect(request, crate::dialect::v0_37::Dialect)
                    .await
            },
            CompatMode::V0_34 => {
                $self
                    .perform_with_dialect(request, crate::dialect::v0_34::Dialect)
                    .await
            },
        }
    }};
}

#[cfg(feature = "http-client")]
pub mod http;
#[cfg(feature = "websocket-client")]
pub mod websocket;
