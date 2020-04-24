use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub use tendermint::hash::Hash;
pub use tendermint::lite::types::Height;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Error {
    HeaderFromTheFuture {
        header_time: SystemTime,
        now: SystemTime,
    },
    ImplementationSpecific,
    InsufficientValidatorsOverlap {
        total_power: u64,
        signed_power: u64,
    },
    InsufficientVotingPower {
        total_power: u64,
        voting_power: u64,
    },
    InvalidCommit {
        total_power: u64,
        signed_power: u64,
    },
    InvalidCommitValue {
        header_hash: Hash,
        commit_hash: Hash,
    },
    InvalidNextValidatorSet {
        header_next_validators_hash: Hash,
        next_validators_hash: Hash,
    },
    InvalidValidatorSet {
        header_validators_hash: Hash,
        validators_hash: Hash,
    },
    NonIncreasingHeight {
        got: Height,
        expected: Height,
    },
    NonMonotonicBftTime {
        header_bft_time: SystemTime,
        trusted_header_bft_time: SystemTime,
    },
    NotWithinTrustPeriod {
        at: SystemTime,
        now: SystemTime,
    },
}

#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct Header {
    pub height: Height,
    pub bft_time: SystemTime,
    pub validators_hash: Hash,
    pub next_validators_hash: Hash,
    pub hash: Hash, // TODO: What if we don't have this
}

#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct ValidatorSet {
    pub hash: Hash,
}

impl From<std::vec::Vec<tendermint::validator::Info>> for ValidatorSet {
    fn from(_vis: std::vec::Vec<tendermint::validator::Info>) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct Commit {
    pub header_hash: Hash,
}

#[derive(Copy, Clone, Debug, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct TrustThreshold {
    pub numerator: u64,
    pub denominator: u64,
}

#[derive(Clone, Debug, Display, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct TrustedState {
    pub header: Header,
    pub validators: ValidatorSet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightBlock {
    pub height: Height,
    pub signed_header: SignedHeader,
    pub validator_set: ValidatorSet,
    pub next_validator_set: ValidatorSet,
}
