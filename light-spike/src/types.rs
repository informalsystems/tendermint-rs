use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub use tendermint::hash::Hash;
pub use tendermint::lite::Height;

use tendermint::{block::signed_header::SignedHeader as TMSignedHeader, lite::Header as _};

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct VerificationOptions {
    pub trust_threshold: TrustThreshold,
    pub trusting_period: Duration,
    pub now: SystemTime,
}

#[derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct Header {
    pub height: Height,
    pub bft_time: SystemTime,
    pub validators_hash: Hash,
    pub next_validators_hash: Hash,
    pub hash: Hash, // TODO: What if we don't have this
}

#[derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct ValidatorSet {
    pub hash: Hash,
}

impl From<std::vec::Vec<tendermint::validator::Info>> for ValidatorSet {
    fn from(_vis: std::vec::Vec<tendermint::validator::Info>) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct Commit {
    pub header_hash: Hash,
    pub commit: tendermint::block::Commit,
}

#[derive(Copy, Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct TrustThreshold {
    pub numerator: u64,
    pub denominator: u64,
}

#[derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct SignedHeader {
    pub header: Header,
    pub commit: Commit,
    pub validators: ValidatorSet,
    pub validators_hash: Hash,
}

impl From<TMSignedHeader> for SignedHeader {
    fn from(sh: TMSignedHeader) -> Self {
        let validators = ValidatorSet {
            hash: sh.header.validators_hash(),
        };

        Self {
            header: Header {
                height: sh.header.height().into(),
                bft_time: sh.header.bft_time().to_system_time().unwrap(),
                validators_hash: sh.header.validators_hash(),
                next_validators_hash: sh.header.next_validators_hash(),
                hash: sh.header.hash(),
            },
            commit: Commit {
                header_hash: sh.header.hash(),
                commit: sh.commit,
            },
            validators: validators.clone(),
            validators_hash: validators.hash,
        }
    }
}

pub type TrustedState = LightBlock;

#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct LightBlock {
    pub height: Height,
    pub signed_header: SignedHeader,
    pub validators: ValidatorSet,
    pub next_validators: ValidatorSet,
}

impl LightBlock {
    pub fn header(&self) -> &Header {
        &self.signed_header.header
    }
}

impl From<tendermint::block::signed_header::SignedHeader> for LightBlock {
    fn from(sh: tendermint::block::signed_header::SignedHeader) -> Self {
        let height = sh.header.height.into();

        let validators = ValidatorSet {
            hash: sh.header.validators_hash(),
        };

        let next_validators = ValidatorSet {
            hash: sh.header.next_validators_hash(),
        };

        let signed_header = sh.into();

        Self {
            height,
            signed_header,
            validators,
            next_validators,
        }
    }
}
