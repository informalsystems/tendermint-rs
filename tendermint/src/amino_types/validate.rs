use thiserror::Error;

pub trait ConsensusMessage {
    fn validate_basic(&self) -> Result<(), Error>;
}

pub type Error = anomaly::BoxError;

/// Kinds of validation errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Error)]
pub enum Kind {
    #[error("invalid Type")]
    InvalidMessageType,
    #[error("overflow Type")]
    OverflowMessageType,
    #[error("consensus message is missing")]
    MissingConsensusMessage,
    #[error("negative height")]
    NegativeHeight,
    #[error("negative round")]
    NegativeRound,
    #[error("round overflow")]
    OverflowRound,
    #[error("negative POLRound (exception: -1)")]
    NegativePOLRound,
    #[error("negative ValidatorIndex")]
    NegativeValidatorIndex,
    #[error("ValidatorIndex overflow")]
    OverflowValidatorIndex,
    #[error("expected ValidatorAddress size to be 20 bytes")]
    InvalidValidatorAddressSize,
    #[error("Wrong hash: expected Hash size to be 32 bytes")]
    InvalidHashSize,
    #[error("negative total")]
    NegativeTotal,
}
