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

pub type PeerId = tendermint::node::Id;

pub type TrustThreshold = TrustThresholdFraction;

pub type Header = TMHeader;

pub type ValidatorSet = TMValidatorSet;

pub type Commit = TMCommit;

pub type SignedHeader = TMSignedHeader;

pub type TrustedState = LightBlock;

// FIXME: Remove when conformance tests are adapted to include provider
fn primary() -> PeerId {
    "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap()
}

#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct LightBlock {
    pub signed_header: SignedHeader,
    #[serde(rename = "validator_set")]
    pub validators: ValidatorSet,
    #[serde(rename = "next_validator_set")]
    pub next_validators: ValidatorSet,
    // FIXME: Remove annotation when conformance tests are adapted to include provider
    #[serde(default = "primary")]
    pub provider: PeerId,
}

impl LightBlock {
    pub fn new(
        signed_header: SignedHeader,
        validators: ValidatorSet,
        next_validators: ValidatorSet,
        provider: PeerId,
    ) -> LightBlock {
        Self {
            signed_header,
            validators,
            next_validators,
            provider,
        }
    }

    pub fn height(&self) -> Height {
        self.signed_header.header.height.into()
    }
}
