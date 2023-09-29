//! Helper types to generalize differences in serialization between
//! Tendermint RPC protocol versions.

pub mod v0_34;
pub mod v0_37;
pub use v0_37::Dialect as LatestDialect;

mod begin_block;
mod check_tx;
mod deliver_tx;
mod end_block;

pub use begin_block::BeginBlock;
pub use check_tx::CheckTx;
pub use deliver_tx::DeliverTx;
pub use end_block::EndBlock;

use serde::{de::DeserializeOwned, Serialize};

use tendermint::{abci, evidence};

pub trait Dialect: sealed::Sealed + Default + Clone + Send + Sync {
    type Event: Into<abci::Event> + Serialize + DeserializeOwned;
    type Evidence: From<evidence::Evidence> + Serialize + DeserializeOwned + Send;
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::v0_34::Dialect {}
    impl Sealed for super::v0_37::Dialect {}
}
