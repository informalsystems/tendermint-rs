//! Defines or just re-exports the main datatypes used by the light client.

use derive_more::Display;
use serde::{Deserialize, Serialize};

use tendermint::{
    account::Id as TMAccountId,
    block::{
        header::Header as TMHeader, signed_header::SignedHeader as TMSignedHeader,
        Commit as TMCommit,
    },
    trust_threshold::TrustThresholdFraction,
    validator::Info as TMValidatorInfo,
    validator::Set as TMValidatorSet,
};

pub use tendermint::{block::Height, hash::Hash, time::Time};

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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Status {
    /// The light block has failed verification.
    Failed,
    /// The light has not been verified yet.
    Unverified,
    /// The light block has been successfully verified.
    Verified,
    /// The light block has been successfully verified and has passed fork detection.
    Trusted,
}

impl Status {
    /// Return a slice of all the possible values for this enum.
    pub fn iter() -> &'static [Self] {
        use Status::*;
        static ALL: &[Status] = &[Unverified, Verified, Trusted, Failed];
        ALL
    }

    /// Returns the most trusted status between the two given one.
    ///
    /// From least to most trusted: `Failed`, `Unverified`, `Verified`, `Trusted`.
    pub fn most_trusted(a: Self, b: Self) -> Self {
        std::cmp::max(a, b)
    }
}

/// Encapsulated reference to the state of a [`LightBlock`] without its
/// provider. Primarily used in Light Client verification.
pub struct LightBlockState<'a> {
    pub signed_header: &'a SignedHeader,
    pub validators: &'a ValidatorSet,
    pub next_validators: &'a ValidatorSet,
}

impl<'a> LightBlockState<'a> {
    /// Convenience method to expose the height of the associated header.
    pub fn height(&self) -> Height {
        self.signed_header.header.height
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
    /// This is a shorthand for `block.signed_header.header.height`.
    pub fn height(&self) -> Height {
        self.signed_header.header.height
    }

    /// Obtain the state of the light block, without its provider.
    pub fn state(&self) -> LightBlockState<'_> {
        LightBlockState {
            signed_header: &self.signed_header,
            validators: &self.validators,
            next_validators: &self.next_validators,
        }
    }
}

/// Contains the local status information, like the latest height, latest block and valset hashes,
/// list of of connected full nodes (primary and witnesses).
#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct LatestStatus {
    /// The latest height we are trusting.
    pub height: Option<u64>,
    /// The latest block hash we are trusting.
    #[serde(with = "tendermint::serializers::option_hash")]
    pub block_hash: Option<Hash>,
    /// The latest validator set we are trusting.
    /// Note that this potentially did not yet sign a header yet.
    #[serde(with = "tendermint::serializers::option_hash")]
    pub valset_hash: Option<Hash>,
    /// The list of fullnodes we are connected to, primary and witnesses.
    pub connected_nodes: Vec<PeerId>,
}

impl LatestStatus {
    /// Builds a new instance of this struct.
    pub fn new(
        height: Option<u64>,
        block_hash: Option<Hash>,
        valset_hash: Option<Hash>,
        connected_nodes: Vec<PeerId>,
    ) -> Self {
        Self {
            height,
            block_hash,
            valset_hash,
            connected_nodes,
        }
    }
}

#[cfg(test)]
mod tests {

    mod status {
        use crate::types::Status;
        use Status::*;

        #[test]
        fn ord_impl() {
            assert!(Trusted > Verified);
            assert!(Verified > Unverified);
            assert!(Unverified > Failed);
        }

        #[test]
        fn most_trusted() {
            for (a, b) in cross(Status::iter()) {
                if a > b {
                    assert_eq!(Status::most_trusted(a, b), a);
                } else {
                    assert_eq!(Status::most_trusted(a, b), b);
                }
            }
        }

        fn cross<T>(xs: &[T]) -> Vec<(T, T)>
        where
            T: Copy,
        {
            xs.iter()
                .copied()
                .flat_map(|y| xs.iter().copied().map(move |x| (x, y)))
                .collect()
        }
    }
}
