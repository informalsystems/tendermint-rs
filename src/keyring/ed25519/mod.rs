pub use signatory::ed25519::{PublicKey, Seed, PUBLIC_KEY_SIZE};

//mod public_key;
mod signer;
#[cfg(feature = "softsign")]
pub mod softsign;
#[cfg(feature = "yubihsm")]
pub mod yubihsm;

pub use self::signer::Signer;
