//! Votes from validators

mod canonical_vote;
mod power;
mod sign_vote;
mod validator_index;

pub use self::canonical_vote::CanonicalVote;
pub use self::power::Power;
pub use self::sign_vote::*;
pub use self::validator_index::ValidatorIndex;

use core::convert::{TryFrom, TryInto};
use core::fmt;
use core::str::FromStr;

use bytes::BufMut;
use serde::{Deserialize, Serialize};

use tendermint_proto::types::Vote as RawVote;
use tendermint_proto::{Error as ProtobufError, Protobuf};

use crate::chain::Id as ChainId;
use crate::consensus::State;
use crate::error::Error;
use crate::hash;
use crate::prelude::*;
use crate::signature::Ed25519Signature;
use crate::{account, block, Signature, Time};

/// Votes are signed messages from validators for a particular block which
/// include information about the validator signing it.
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#vote>
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(try_from = "RawVote", into = "RawVote")]
pub struct Vote {
    /// Type of vote (prevote or precommit)
    pub vote_type: Type,

    /// Block height
    pub height: block::Height,

    /// Round
    pub round: block::Round,

    /// Block ID
    pub block_id: Option<block::Id>,

    /// Timestamp
    pub timestamp: Option<Time>,

    /// Validator address
    pub validator_address: account::Id,

    /// Validator index
    pub validator_index: ValidatorIndex,

    /// Signature
    pub signature: Option<Signature>,
}

impl Protobuf<RawVote> for Vote {}

impl TryFrom<RawVote> for Vote {
    type Error = Error;

    fn try_from(value: RawVote) -> Result<Self, Self::Error> {
        if value.timestamp.is_none() {
            return Err(Error::missing_timestamp());
        }
        Ok(Vote {
            vote_type: value.r#type.try_into()?,
            height: value.height.try_into()?,
            round: value.round.try_into()?,
            // block_id can be nil in the Go implementation
            block_id: value
                .block_id
                .map(TryInto::try_into)
                .transpose()?
                .filter(|i| i != &block::Id::default()),
            timestamp: value.timestamp.map(|t| t.try_into()).transpose()?,
            validator_address: value.validator_address.try_into()?,
            validator_index: value.validator_index.try_into()?,
            signature: Signature::new(value.signature)?,
        })
    }
}

impl From<Vote> for RawVote {
    fn from(value: Vote) -> Self {
        RawVote {
            r#type: value.vote_type.into(),
            height: value.height.into(),
            round: value.round.into(),
            block_id: value.block_id.map(Into::into),
            timestamp: value.timestamp.map(Into::into),
            validator_address: value.validator_address.into(),
            validator_index: value.validator_index.into(),
            signature: value.signature.map(|s| s.to_bytes()).unwrap_or_default(),
        }
    }
}

impl Vote {
    /// Is this vote a prevote?
    pub fn is_prevote(&self) -> bool {
        match self.vote_type {
            Type::Prevote => true,
            Type::Precommit => false,
        }
    }

    /// Is this vote a precommit?
    pub fn is_precommit(&self) -> bool {
        match self.vote_type {
            Type::Precommit => true,
            Type::Prevote => false,
        }
    }

    /// Returns block_id.hash
    pub fn header_hash(&self) -> Option<hash::Hash> {
        self.block_id.map(|b| b.hash)
    }

    /// Create signable bytes from Vote.
    pub fn to_signable_bytes<B>(
        &self,
        chain_id: ChainId,
        sign_bytes: &mut B,
    ) -> Result<bool, ProtobufError>
    where
        B: BufMut,
    {
        CanonicalVote::new(self.clone(), chain_id).encode_length_delimited(sign_bytes)?;
        Ok(true)
    }

    /// Create signable vector from Vote.
    pub fn to_signable_vec(&self, chain_id: ChainId) -> Result<Vec<u8>, ProtobufError> {
        CanonicalVote::new(self.clone(), chain_id).encode_length_delimited_vec()
    }

    /// Consensus state from this vote - This doesn't seem to be used anywhere.
    #[deprecated(
        since = "0.17.0",
        note = "This seems unnecessary, please raise it to the team, if you need it."
    )]
    pub fn consensus_state(&self) -> State {
        State {
            height: self.height,
            round: self.round,
            step: 6,
            block_id: self.block_id,
        }
    }
}

/// Default trait. Used in tests.
// FIXME: Does it need to be in public crate API? If not, replace with a helper fn in crate::test?
impl Default for Vote {
    fn default() -> Self {
        Vote {
            vote_type: Type::Prevote,
            height: Default::default(),
            round: Default::default(),
            block_id: None,
            timestamp: Some(Time::unix_epoch()),
            validator_address: account::Id::new([0; account::LENGTH]),
            validator_index: ValidatorIndex::try_from(0_i32).unwrap(),
            // Could have reused crate::test::dummy_signature, except that
            // this Default impl is defined outside of #[cfg(test)].
            signature: Some(Signature::from(
                Ed25519Signature::from_bytes(&[0; Ed25519Signature::BYTE_SIZE]).unwrap(),
            )),
        }
    }
}
/// SignedVote is the union of a canonicalized vote, the signature on
/// the sign bytes of that vote and the id of the validator who signed it.
pub struct SignedVote {
    vote: CanonicalVote,
    validator_address: account::Id,
    signature: Signature,
}

impl SignedVote {
    /// Create new `SignedVote` from provided canonicalized vote, validator id, and
    /// the signature of that validator.
    pub fn new(
        vote: Vote,
        chain_id: ChainId,
        validator_address: account::Id,
        signature: Signature,
    ) -> SignedVote {
        let canonical_vote = CanonicalVote::new(vote, chain_id);
        SignedVote {
            vote: canonical_vote,
            signature,
            validator_address,
        }
    }

    /// Create a new `SignedVote` from the provided `Vote`, which may or may not be signed.
    /// If the vote is not signed, this function will return `None`.
    pub fn from_vote(vote: Vote, chain_id: ChainId) -> Option<Self> {
        let validator_address = vote.validator_address;
        vote.signature
            .clone()
            .map(|signature| Self::new(vote, chain_id, validator_address, signature))
    }

    /// Return the id of the validator that signed this vote.
    pub fn validator_id(&self) -> account::Id {
        self.validator_address
    }

    /// Return the bytes (of the canonicalized vote) that were signed.
    pub fn sign_bytes(&self) -> Vec<u8> {
        self.vote.encode_length_delimited_vec().unwrap()
    }

    /// Return the actual signature on the canonicalized vote.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

/// Types of votes
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    /// Votes for blocks which validators observe are valid for a given round
    Prevote = 1,

    /// Votes to commit to a particular block for a given round
    Precommit = 2,
}

impl Protobuf<i32> for Type {}

impl TryFrom<i32> for Type {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Type::Prevote),
            2 => Ok(Type::Precommit),
            _ => Err(Error::invalid_message_type()),
        }
    }
}

impl From<Type> for i32 {
    fn from(value: Type) -> Self {
        value as i32
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = match self {
            Type::Prevote => "Prevote",
            Type::Precommit => "Precommit",
        };
        write!(f, "{}", id)
    }
}

impl FromStr for Type {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Prevote" => Ok(Self::Prevote),
            "Precommit" => Ok(Self::Precommit),
            _ => Err(Error::invalid_message_type()),
        }
    }
}
