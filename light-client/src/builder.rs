//! DSL for building light clients and supervisor

mod light_client;
pub use light_client::LightClientBuilder;

#[cfg(feature = "rpc-client")]
mod supervisor;

#[cfg(feature = "rpc-client")]
pub use supervisor::SupervisorBuilder;

pub mod error;
