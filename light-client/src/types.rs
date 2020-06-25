//! Defines or just re-exports the main datatypes used by the light client.

use derive_more::Display;
use serde::{Deserialize, Serialize};

use tendermint::{
    account::Id as TMAccountId,
    block::{
        header::Header as TMHeader, signed_header::SignedHeader as TMSignedHeader,
        Commit as TMCommit,
    },
    lite::TrustThresholdFraction,
    validator::Info as TMValidatorInfo,
    validator::Set as TMValidatorSet,
};

pub use tendermint::{hash::Hash, lite::Height, time::Time};

/// Peer ID (public key) of a full node
pub type PeerId = tendermint::node::Id;

/// defines what fraction of the total voting power of a known
/// and trusted validator set is sufficient for a commit to be
/// accepted going forward.
pub type TrustThreshold = TrustThresholdFraction;

/// A header contains metadata about the block and about the
/// consensus, as well as commitments to the data in the current block, the
/// previous block, and the results returned by the application.
pub type Header = TMHeader;

/// Set of validators
pub type ValidatorSet = TMValidatorSet;

/// Info about a single validator
pub type Validator = TMValidatorInfo;

/// Validator address
pub type ValidatorAddress = TMAccountId;

/// A commit contains the justification (ie. a set of signatures)
/// that a block was consensus, as committed by a set previous block of validators.
pub type Commit = TMCommit;

/// A signed header contains both a `Header` and its corresponding `Commit`.
pub type SignedHeader = TMSignedHeader;

/// A type alias for a `LightBlock`.
pub type TrustedState = LightBlock;

/// Verification status of a light block.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// The light has not been verified yet.
    Unverified,
    /// The light block has been successfully verified.
    Verified,
    /// The light block has been successfully verified and has passed fork detection.
    Trusted,
    /// The light block has failed verification.
    Failed,
}

impl Status {
    /// Return a slice of all the possible values for this enum.
    pub fn iter() -> &'static [Status] {
        static ALL: &[Status] = &[
            Status::Unverified,
            Status::Verified,
            Status::Trusted,
            Status::Failed,
        ];

        ALL
    }
}

/// A light block is the core data structure used by the light client.
/// It records everything the light client needs to know about a block.
#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct LightBlock {
    /// Header and commit of this block
    pub signed_header: SignedHeader,
    /// Validator set at the block height
    #[serde(rename = "validator_set")]
    pub validators: ValidatorSet,
    /// Validator set at the next block height
    #[serde(rename = "next_validator_set")]
    pub next_validators: ValidatorSet,
    /// The peer ID of the node that provided this block
    pub provider: PeerId,
}

impl LightBlock {
    /// Constructs a new light block
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

    /// Returns the height of this block.
    ///
    /// ## Note
    /// This is a shorthand for `block.signed_header.header.height.into()`.
    pub fn height(&self) -> Height {
        self.signed_header.header.height.into()
    }
}
