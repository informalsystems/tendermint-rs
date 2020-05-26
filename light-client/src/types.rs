use derive_more::Display;
use serde::{Deserialize, Serialize};

use tendermint::{
    block::{
        header::Header as TMHeader, signed_header::SignedHeader as TMSignedHeader,
        Commit as TMCommit,
    },
    lite::TrustThresholdFraction,
    validator::Set as TMValidatorSet,
};

pub use tendermint::{hash::Hash, lite::Height, time::Time};

pub type PeerId = tendermint::node::Id;

pub type TrustThreshold = TrustThresholdFraction;

pub type Header = TMHeader;

pub type ValidatorSet = TMValidatorSet;

pub type Commit = TMCommit;

pub type SignedHeader = TMSignedHeader;

pub type TrustedState = LightBlock;

#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct LightBlock {
    pub signed_header: SignedHeader,
    #[serde(rename = "validator_set")]
    pub validators: ValidatorSet,
    #[serde(rename = "next_validator_set")]
    pub next_validators: ValidatorSet,
    // FIXME: Uncomment when conformance tests are adapted to include provider
    // pub provider: PeerId,
}

impl LightBlock {
    pub fn new(
        signed_header: SignedHeader,
        validators: ValidatorSet,
        next_validators: ValidatorSet,
        // provider: PeerId,
    ) -> LightBlock {
        Self {
            signed_header,
            validators,
            next_validators,
            // provider,
        }
    }

    pub fn height(&self) -> Height {
        self.signed_header.header.height.into()
    }
}
