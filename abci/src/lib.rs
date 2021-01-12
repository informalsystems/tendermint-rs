//! ABCI framework for building applications with Tendermint.

mod application;
mod codec;
mod result;

pub mod client;
pub mod server;

pub use application::Application;
pub use result::{Error, Result};
