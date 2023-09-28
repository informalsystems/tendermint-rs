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
                    .execute::<_, crate::dialect::v0_37::Dialect>(request)
                    .await
            },
            CompatMode::V0_34 => {
                $self
                    .execute::<_, crate::dialect::v0_34::Dialect>(request)
                    .await
            },
        }
    }};
}

#[cfg(feature = "http-client")]
pub mod http;
#[cfg(feature = "websocket-client")]
pub mod websocket;
