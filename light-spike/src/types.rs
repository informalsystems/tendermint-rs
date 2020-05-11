use derive_more::Display;
use serde::{Deserialize, Serialize};

use tendermint::{
    block::{
        header::Header as TMHeader, signed_header::SignedHeader as TMSignedHeader,
        Commit as TMCommit,
    },
    validator::Set as TMValidatorSet,
    Time,
};

pub use tendermint::{hash::Hash, lite::Height};

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct VerificationOptions {
    pub trust_threshold: TrustThreshold,
    pub trusting_period: Duration,
    pub now: Time,
}

impl VerificationOptions {
    pub fn with_now(&self, now: Time) -> Self {
        Self {
            now,
            ..self.clone()
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct TrustThreshold {
    pub numerator: u64,
    pub denominator: u64,
}

pub type Header = TMHeader;

pub type ValidatorSet = TMValidatorSet;

pub type Commit = TMCommit;

pub type SignedHeader = TMSignedHeader;

pub type TrustedState = LightBlock;

#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct LightBlock {
    pub height: Height,
    pub signed_header: SignedHeader,
    pub validators: ValidatorSet,
    pub next_validators: ValidatorSet,
    pub provider: Peer,
}

impl LightBlock {
    pub fn new(
        sh: SignedHeader,
        validators: ValidatorSet,
        next_validators: ValidatorSet,
        provider: Peer,
    ) -> LightBlock {
        let height = sh.header.height.into();

        let signed_header = sh.into();

        Self {
            height,
            signed_header,
            validators,
            next_validators,
            provider,
        }
    }
}
