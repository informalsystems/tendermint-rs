use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub use tendermint::hash::Hash;
pub use tendermint::lite::types::Height;

#[derive(Clone, Debug, PartialEq, Eq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct Header {
    pub height: Height,
    pub bft_time: SystemTime,
    pub validators_hash: Hash,
    pub next_validators_hash: Hash,
    pub hash: Hash, // TODO: What if we don't have this
}

#[derive(Clone, Debug, PartialEq, Eq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct ValidatorSet {
    pub hash: Hash,
}

impl From<std::vec::Vec<tendermint::validator::Info>> for ValidatorSet {
    fn from(_vis: std::vec::Vec<tendermint::validator::Info>) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct Commit {
    pub header_hash: Hash,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct TrustThreshold {
    pub numerator: u64,
    pub denominator: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct SignedHeader {
    pub header: Header,
    pub commit: Commit,
    pub validators: ValidatorSet,
    pub validators_hash: Hash,
}

impl From<tendermint::block::signed_header::SignedHeader> for SignedHeader {
    fn from(_sh: tendermint::block::signed_header::SignedHeader) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct TrustedState {
    pub header: Header,
    pub validators: ValidatorSet,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightBlock {
    pub height: Height,
    pub signed_header: SignedHeader,
    pub validator_set: ValidatorSet,
    pub next_validator_set: ValidatorSet,
}
