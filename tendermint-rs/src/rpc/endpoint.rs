//! Tendermint JSONRPC endpoints

mod net_info;
mod status;

pub use net_info::{NetInfoRequest, NetInfoResponse};
pub use status::{StatusRequest, StatusResponse};
