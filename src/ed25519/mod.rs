pub use signatory::ed25519::Seed;
use subtle_encoding::{Identity, IDENTITY};

pub mod keyring;
mod public_key;
mod signer;

pub use self::keyring::KeyRing;
pub use self::public_key::{ConsensusKey, PublicKey, PUBLIC_KEY_SIZE};
pub use self::signer::Signer;

/// Encoding for secret keys
pub const SECRET_KEY_ENCODING: &Identity = IDENTITY;
