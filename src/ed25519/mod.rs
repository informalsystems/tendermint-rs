use subtle_encoding;

pub mod keyring;
mod public_key;
mod signer;

pub use self::keyring::KeyRing;
pub use self::public_key::{PublicKey, PUBLIC_KEY_SIZE};
pub use self::signer::Signer;

/// Encoding for secret keys
pub const SECRET_KEY_ENCODING: &subtle_encoding::Identity = subtle_encoding::IDENTITY;
