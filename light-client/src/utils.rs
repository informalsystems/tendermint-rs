//! Various general-purpose utilities

#[cfg(feature = "rpc-client")]
pub mod block_on;
#[cfg(feature = "rpc-client")]
pub use block_on::block_on;

pub mod std_ext;
