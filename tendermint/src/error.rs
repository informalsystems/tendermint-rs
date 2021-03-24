//! Error types

use anomaly::{BoxError, Context};
use thiserror::Error;

use crate::account;
use crate::vote;

/// Error type
pub type Error = BoxError;

/// Kinds of errors
#[derive(Clone, Eq, PartialEq, Debug, Error)]
pub enum Kind {
    /// Cryptographic operation failed
    #[error("cryptographic error")]
    Crypto,

    /// Malformatted or otherwise invalid cryptographic key
    #[error("invalid key")]
    InvalidKey,

    /// Input/output error
    #[error("I/O error")]
    Io,

    /// Length incorrect or too long
    #[error("length error")]
    Length,

    /// Parse error
    #[error("parse error")]
    Parse,

    /// Network protocol-related errors
    #[error("protocol error")]
    Protocol,

    /// Value out-of-range
    #[error("value out of range")]
    OutOfRange,

    /// Signature invalid
    #[error("bad signature")]
    SignatureInvalid,

    /// invalid message type
    #[error("invalid message type")]
    InvalidMessageType,

    /// Negative block height
    #[error("negative height")]
    NegativeHeight,

    /// Negative voting round
    #[error("negative round")]
    NegativeRound,

    /// Negative POL round
    #[error("negative POL round")]
    NegativePolRound,

    /// Negative validator index in vote
    #[error("negative validator index")]
    NegativeValidatorIndex,

    /// Invalid hash size in part_set_header
    #[error("invalid hash: expected hash size to be 32 bytes")]
    InvalidHashSize,

    /// No timestamp in vote or block header
    #[error("no timestamp")]
    NoTimestamp,

    /// Invalid timestamp
    #[error("invalid timestamp")]
    InvalidTimestamp,

    /// Invalid account ID length
    #[error("invalid account ID length")]
    InvalidAccountIdLength,

    /// Invalid signature ID length
    #[error("invalid signature ID length")]
    InvalidSignatureIdLength,

    /// Overflow during conversion
    #[error("integer overflow")]
    IntegerOverflow,

    /// No Vote found during conversion
    #[error("no vote found")]
    NoVoteFound,

    /// No Proposal found during conversion
    #[error("no proposal found")]
    NoProposalFound,

    /// Invalid AppHash length found during conversion
    #[error("invalid app hash Length")]
    InvalidAppHashLength,

    /// Invalid PartSetHeader
    #[error("invalid part set header")]
    InvalidPartSetHeader,

    /// Missing Header in Block
    #[error("missing header field")]
    MissingHeader,

    /// Missing Data in Block
    #[error("missing data field")]
    MissingData,

    /// Missing Evidence in Block
    #[error("missing evidence field")]
    MissingEvidence,

    /// Missing Timestamp in Block
    #[error("missing timestamp field")]
    MissingTimestamp,

    /// Invalid Block
    #[error("invalid block")]
    InvalidBlock,

    /// Invalid first Block
    #[error("invalid first block")]
    InvalidFirstBlock,

    /// Missing Version field
    #[error("missing version")]
    MissingVersion,

    /// Invalid Header
    #[error("invalid header")]
    InvalidHeader,

    /// Invalid first Header
    #[error("invalid first header")]
    InvalidFirstHeader,

    /// Invalid signature in CommitSig
    #[error("invalid signature")]
    InvalidSignature,

    /// Invalid validator address in CommitSig
    #[error("invalid validator address")]
    InvalidValidatorAddress,

    /// Invalid Signed Header
    #[error("invalid signed header")]
    InvalidSignedHeader,

    /// Invalid Evidence
    #[error("invalid evidence")]
    InvalidEvidence,

    /// Invalid BlockIdFlag
    #[error("invalid block id flag")]
    BlockIdFlag,

    /// Negative voting power
    #[error("negative power")]
    NegativePower,

    /// Mismatch between raw voting power and computed one in validator set
    #[error("mismatch between raw voting power ({raw}) and computed one ({computed})")]
    RawVotingPowerMismatch {
        /// raw voting power
        raw: vote::Power,
        /// computed voting power
        computed: vote::Power,
    },

    /// Missing Public Key
    #[error("missing public key")]
    MissingPublicKey,

    /// Invalid validator parameters
    #[error("invalid validator parameters")]
    InvalidValidatorParams,

    /// Invalid version parameters
    #[error("invalid version parameters")]
    InvalidVersionParams,

    /// Negative max_age_num_blocks in Evidence parameters
    #[error("negative max_age_num_blocks")]
    NegativeMaxAgeNum,

    /// Missing max_age_duration in evidence parameters
    #[error("missing max_age_duration")]
    MissingMaxAgeDuration,

    /// Proposer not found in validator set
    #[error("proposer with address '{}' not found in validator set", _0)]
    ProposerNotFound(account::Id),
}

impl Kind {
    /// Add additional context.
    pub fn context(self, source: impl Into<BoxError>) -> Context<Kind> {
        Context::new(self, Some(source.into()))
    }
}
