use core::fmt::{self, Display};

use crate::{PublicKey, Signature};

/// Signature error.
///
#[derive(Debug)]
pub enum Error {
    /// This variant is deliberately opaque as to avoid side-channel leakage.
    VerificationFailed,
    /// The key used to verify a signature is not of a type supported by the implementation.
    UnsupportedKeyType,
    /// The encoding of the public key was malformed.
    MalformedPublicKey,
    /// The signature data was malformed.
    MalformedSignature,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::VerificationFailed => f.write_str("invalid signature"),
            Error::UnsupportedKeyType => f.write_str("key type not supported"),
            Error::MalformedPublicKey => f.write_str("malformed public key encoding"),
            Error::MalformedSignature => f.write_str("malformed signature"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

pub trait Verifier {
    fn verify(pubkey: PublicKey, msg: &[u8], signature: &Signature) -> Result<(), Error>;
}
