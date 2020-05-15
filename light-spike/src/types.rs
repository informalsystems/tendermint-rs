use derive_more::Display;
use serde::{Deserialize, Serialize};

use tendermint::{
    block::{
        header::Header as TMHeader, signed_header::SignedHeader as TMSignedHeader,
        Commit as TMCommit,
    },
    lite::TrustThresholdFraction,
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

pub type Peer = tendermint::net::Address;

pub type TrustThreshold = TrustThresholdFraction;

pub type Header = TMHeader;

pub type ValidatorSet = TMValidatorSet;

pub type Commit = TMCommit;

pub type SignedHeader = TMSignedHeader;

pub type TrustedState = LightBlock;

fn primary() -> Peer {
    "tcp://localhost:1337".parse().unwrap()
}

#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct LightBlock {
    // pub height: Height,
    pub signed_header: SignedHeader,
    #[serde(rename = "validator_set")]
    pub validators: ValidatorSet,
    #[serde(rename = "next_validator_set")]
    pub next_validators: ValidatorSet,
    #[serde(default = "primary")]
    pub provider: Peer,
}

impl LightBlock {
    pub fn new(
        sh: SignedHeader,
        validators: ValidatorSet,
        next_validators: ValidatorSet,
        provider: Peer,
    ) -> LightBlock {
        // let height = sh.header.height.into();
        Self {
            // height,
            signed_header: sh.into(),
            validators,
            next_validators,
            provider,
        }
    }

    pub fn height(&self) -> Height {
        self.signed_header.header.height.into()
    }
}
