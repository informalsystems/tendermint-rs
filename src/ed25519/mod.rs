use signatory;

pub mod keyring;
mod public_key;
mod signer;

pub use self::keyring::KeyRing;
pub use self::public_key::{PublicKey, PUBLIC_KEY_SIZE};
pub use self::signer::Signer;
pub use self::keyring::GLOBAL_KEYRING;

/// Encoding for secret keys
pub const SECRET_KEY_ENCODING: signatory::Encoding = signatory::Encoding::Raw;
