//! Old ABCI structures, formerly defined in `tendermint::abci`.
//!
//! The original contents of `tendermint::abci` were created only to model RPC
//! responses, not to model ABCI itself:
//!
//! > NOTE: This module contains types for ABCI responses as consumed from RPC
//! endpoints. It does not contain an ABCI protocol implementation.
//!
//! The old types should be eliminated and
//! merged with the new ABCI domain types.  Moving them here in the meantime
//! disentangles improving the ABCI domain modeling from changes to the RPC
//! interface.

mod code;
mod data;
mod gas;
mod info;
mod log;
mod path;

pub mod responses;
pub mod tag;
pub mod transaction;

pub use self::{
    code::Code,
    data::Data,
    gas::Gas,
    info::Info,
    log::Log,
    path::Path,
    responses::{DeliverTx, Event, Responses},
    transaction::Transaction,
};
