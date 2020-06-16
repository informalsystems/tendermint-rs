//! Tendermint RPC: JSONRPC over HTTP support
//!
//! Wraps the RPC API described at: <https://tendermint.com/rpc/>

mod client;
pub mod endpoint;
pub mod error;
// TODO(ismail): document fields or re-use the abci types
#[allow(missing_docs)]
pub mod event_listener;
mod id;
mod method;
mod proxy;
pub mod request;
pub mod response;
mod version;

pub use self::{
    client::Client, error::Error, id::Id, method::Method, request::Request, response::Response,
    version::Version,
};
