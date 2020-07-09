//! Defines or just re-exports the main datatypes used by the light client.

use std::fmt::Debug;

use derive_more::Display;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use tendermint::{
    account::Id as TMAccountId, evidence::Evidence as TMEvidence, lite::TrustThresholdFraction,
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
pub type TMHeader = tendermint::block::Header;

/// Set of validators
pub type TMValidatorSet = tendermint::validator::Set;

/// Info about a single validator
pub type TMValidatorInfo = tendermint::validator::Info;

/// Validator address
pub type TMValidatorAddress = TMAccountId;

/// A commit contains the justification (ie. a set of signatures)
/// that a block was consensus, as committed by a set previous block of validators.
pub type TMCommit = tendermint::block::Commit;

/// A signed header contains both a `Header` and its corresponding `Commit`.
pub type TMSignedHeader = tendermint::block::signed_header::SignedHeader;

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

    pub fn most_trusted(a: Self, b: Self) -> Self {
        std::cmp::max(a, b)
    }
}

pub trait Commit {
    fn block_hash(&self) -> Hash;
}

pub trait LightBlock: Send + Sync + Clone + Debug + Serialize + DeserializeOwned + 'static {
    type Header;
    type Commit: Commit;
    type SignedHeader: Clone;
    type ValidatorSet;
    type Evidence;

    fn height(&self) -> Height;
    fn signed_header(&self) -> &Self::SignedHeader;
    fn header(&self) -> &Self::Header;
    fn commit(&self) -> &Self::Commit;
    fn validators(&self) -> &Self::ValidatorSet;
    fn next_validators(&self) -> &Self::ValidatorSet;
    fn header_time(&self) -> Time;
    fn validators_hash(&self) -> Hash;
    fn next_validators_hash(&self) -> Hash;
    fn provider(&self) -> PeerId;
}

impl Commit for TMCommit {
    fn block_hash(&self) -> Hash {
        self.block_id.hash
    }
}

impl LightBlock for TMLightBlock {
    type Header = TMHeader;
    type Commit = TMCommit;
    type SignedHeader = TMSignedHeader;
    type ValidatorSet = TMValidatorSet;
    type Evidence = TMEvidence;

    fn height(&self) -> Height {
        self.signed_header.header.height.into()
    }

    fn signed_header(&self) -> &Self::SignedHeader {
        &self.signed_header
    }

    fn header(&self) -> &Self::Header {
        &self.signed_header.header
    }

    fn commit(&self) -> &Self::Commit {
        &self.signed_header.commit
    }

    fn validators(&self) -> &Self::ValidatorSet {
        &self.validators
    }

    fn next_validators(&self) -> &Self::ValidatorSet {
        &self.next_validators
    }

    fn header_time(&self) -> Time {
        self.header().time
    }

    fn validators_hash(&self) -> Hash {
        self.signed_header.header.validators_hash
    }

    fn next_validators_hash(&self) -> Hash {
        self.signed_header.header.next_validators_hash
    }

    fn provider(&self) -> PeerId {
        self.provider
    }
}

/// A light block is the core data structure used by the light client.
/// It records everything the light client needs to know about a block.
#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct TMLightBlock {
    /// Header and commit of this block
    pub signed_header: TMSignedHeader,
    /// Validator set at the block height
    #[serde(rename = "validator_set")]
    pub validators: TMValidatorSet,
    /// Validator set at the next block height
    #[serde(rename = "next_validator_set")]
    pub next_validators: TMValidatorSet,
    /// The peer ID of the node that provided this block
    pub provider: PeerId,
}

impl TMLightBlock {
    /// Constructs a new light block
    pub fn new(
        signed_header: TMSignedHeader,
        validators: TMValidatorSet,
        next_validators: TMValidatorSet,
        provider: PeerId,
    ) -> Self {
        Self {
            signed_header,
            validators,
            next_validators,
            provider,
        }
    }
}

/// Contains the local status information, like the latest height, latest block and valset hashes,
/// list of of connected full nodes (primary and witnesses).
#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct LatestStatus {
    /// The latest height we are trusting.
    pub height: Option<Height>,
    /// The latest block hash we are trusting.
    pub block_hash: Option<Hash>,
    /// The latest validator set we are trusting.
    /// Note that this potentially did not yet sign a header yet.
    pub valset_hash: Option<Hash>,
    /// The list of fullnodes we are connected to, primary and witnesses.
    pub connected_nodes: Vec<PeerId>,
}

impl LatestStatus {
    pub fn new(
        height: Option<u64>,
        block_hash: Option<Hash>,
        valset_hash: Option<Hash>,
        connected_nodes: Vec<PeerId>,
    ) -> Self {
        LatestStatus {
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
