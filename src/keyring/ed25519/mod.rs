pub use signatory::ed25519::{PublicKey, Seed, PUBLIC_KEY_SIZE};

mod signer;
#[cfg(feature = "softsign")]
pub mod softsign;
#[cfg(feature = "yubihsm")]
pub mod yubihsm;
#[cfg(feature = "ledger")]
pub mod ledger;

pub use self::signer::Signer;
