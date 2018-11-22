pub trait ConsensusMessage {
    fn validate_basic(&self) -> Result<(), ValidationError>;
}

pub struct ValidationError(ValidationErrorKind);

impl ValidationError {
    pub fn new(kind: ValidationErrorKind) -> ValidationError {
        ValidationError(kind)
    }
}

/// Kinds of validation errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ValidationErrorKind {
    #[fail(display = "invalid Type")]
    InvalidMessageType,
    #[fail(display = "negative height")]
    NegativeHeight,
    #[fail(display = "negative round")]
    NegativeRound,
    #[fail(display = "negative POLRound (exception: -1)")]
    NegativePOLRound,
    // TODO validate block ID
}

impl ToString for ValidationError {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}
