//! Helper types to generalize differences in serialization between
//! Tendermint RPC protocol versions.

mod begin_block;
mod check_tx;
mod deliver_tx;
mod end_block;

pub use begin_block::BeginBlock;
pub use check_tx::CheckTx;
pub use deliver_tx::DeliverTx;
pub use end_block::EndBlock;

use serde::{de::DeserializeOwned, Serialize};

pub trait Dialect: sealed::Sealed + Default {
    type Event: Serialize + DeserializeOwned;
}

pub type DefaultDialect = crate::v0_37::Dialect;

mod sealed {
    pub trait Sealed {}

    impl Sealed for crate::v0_34::Dialect {}
    impl Sealed for crate::v0_37::Dialect {}
}
