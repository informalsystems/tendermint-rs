//! Error types

use displaydoc::Display;

use crate::account;
use crate::vote;

/// Error type
pub type Error = anyhow::Error;

#[cfg(feature = "std")]
impl std::error::Error for Kind {}

/// Kinds of errors
#[derive(Clone, Eq, PartialEq, Debug, Display)]
pub enum Kind {
    // Cryptographic operation failed
    /// cryptographic error
    Crypto,

    // Malformatted or otherwise invalid cryptographic key
    /// invalid key
    InvalidKey,

    // Input/output error
    /// I/O error
    Io,

    // Length incorrect or too long
    /// length error
    Length,

    // Parse error
    /// parse error
    Parse,

    // Network protocol-related errors
    /// protocol error
    Protocol,

    // Value out-of-range
    /// value out of range
    OutOfRange,

    // Signature invalid
    /// bad signature
    SignatureInvalid,

    // invalid message type
    /// invalid message type
    InvalidMessageType,

    // Negative block height
    /// negative height
    NegativeHeight,

    // Negative voting round
    /// negative round
    NegativeRound,

    // Negative POL round
    /// negative POL round
    NegativePolRound,

    // Negative validator index in vote
    /// negative validator index
    NegativeValidatorIndex,

    // Invalid hash size in part_set_header
    /// invalid hash: expected hash size to be 32 bytes
    InvalidHashSize,

    // No timestamp in vote or block header
    /// no timestamp
    NoTimestamp,

    // Invalid timestamp
    /// invalid timestamp
    InvalidTimestamp,

    // Invalid account ID length
    /// invalid account ID length
    InvalidAccountIdLength,

    // Invalid signature ID length
    /// invalid signature ID length
    InvalidSignatureIdLength,

    // Overflow during conversion
    /// integer overflow
    IntegerOverflow,

    // No Vote found during conversion
    /// no vote found
    NoVoteFound,

    // No Proposal found during conversion
    /// no proposal found
    NoProposalFound,

    // Invalid AppHash length found during conversion
    /// invalid app hash Length
    InvalidAppHashLength,

    // Invalid PartSetHeader
    /// invalid part set header
    InvalidPartSetHeader,

    // Missing Header in Block
    /// missing header field
    MissingHeader,

    // Missing Data in Block
    /// missing data field
    MissingData,

    // Missing Evidence in Block
    /// missing evidence field
    MissingEvidence,

    // Missing Timestamp in Block
    /// missing timestamp field
    MissingTimestamp,

    // Invalid Block
    /// invalid block
    InvalidBlock,

    // Invalid first Block
    /// invalid first block
    InvalidFirstBlock,

    // Missing Version field
    /// missing version
    MissingVersion,

    // Invalid Header
    /// invalid header
    InvalidHeader,

    // Invalid first Header
    /// invalid first header
    InvalidFirstHeader,

    // Invalid signature in CommitSig
    /// invalid signature
    InvalidSignature,

    // Invalid validator address in CommitSig
    /// invalid validator address
    InvalidValidatorAddress,

    // Invalid Signed Header
    /// invalid signed header
    InvalidSignedHeader,

    // Invalid Evidence
    /// invalid evidence
    InvalidEvidence,

    // Invalid BlockIdFlag
    /// invalid block id flag
    BlockIdFlag,

    // Negative voting power
    /// negative power
    NegativePower,

    // Mismatch between raw voting power and computed one in validator set
    /// mismatch between raw voting power ({raw:?}) and computed one ({computed:?})
    RawVotingPowerMismatch {
        /// raw voting power
        raw: vote::Power,
        /// computed voting power
        computed: vote::Power,
    },

    // Missing Public Key
    /// missing public key
    MissingPublicKey,

    // Invalid validator parameters
    /// invalid validator parameters
    InvalidValidatorParams,

    // Invalid version parameters
    /// invalid version parameters
    InvalidVersionParams,

    // Negative max_age_num_blocks in Evidence parameters
    /// negative max_age_num_blocks
    NegativeMaxAgeNum,

    // Missing max_age_duration in evidence parameters
    /// missing max_age_duration
    MissingMaxAgeDuration,

    // Proposer not found in validator set
    /// proposer with address '{0}' not found in validator set
    ProposerNotFound(account::Id),
}

// impl Kind {
//     /// Add additional context.
//     pub fn context(self, source: impl Into<BoxError>) -> Context<Kind> {
//         Context::new(self, Some(source.into()))
//     }
// }
