use failure::*;

pub trait ConsensusMessage {
    fn validate_basic(&self) -> Result<(), ValidationError>;
}

pub struct ValidationError(ValidationErrorKind);

/// Kinds of validation errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ValidationErrorKind {
    #[fail(display = "invalid Type")]
    InvalidMessageType,
    #[fail(display = "consensus message is missing")]
    MissingConsensusMessage,
    #[fail(display = "negative height")]
    NegativeHeight,
    #[fail(display = "negative round")]
    NegativeRound,
    #[fail(display = "negative POLRound (exception: -1)")]
    NegativePOLRound,
    #[fail(display = "negative ValidatorIndex")]
    NegativeValidatorIndex,
    #[fail(display = "expected ValidatorAddress size to be 20 bytes")]
    InvalidValidatorAddressSize,
    #[fail(display = "Wrong hash: expected Hash size to be 32 bytes")]
    InvalidHashSize,
    #[fail(display = "negative total")]
    NegativeTotal,
}

impl ToString for ValidationError {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<ValidationErrorKind> for ValidationError {
    fn from(kind: ValidationErrorKind) -> Self {
        ValidationError(kind)
    }
}
