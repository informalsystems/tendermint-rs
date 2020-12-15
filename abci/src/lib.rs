//! ABCI framework for building applications with Tendermint.

mod application;
mod codec;
mod protocol;
mod result;

#[cfg(feature = "server")]
pub use application::Application;
#[cfg(feature = "server")]
pub mod server;

pub use result::{Error, Result};
