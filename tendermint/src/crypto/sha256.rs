/// Length of a SHA256 hash in bytes.
pub const HASH_SIZE: usize = 32;

/// A SHA256 digest implementation.
///
/// This trait provides the most general possible interface that can be
/// implemented by host functions in popular on-chain smart contract
/// environments. As such, in can only do one-piece slice digests.
pub trait Sha256 {
    fn digest(data: impl AsRef<[u8]>) -> [u8; HASH_SIZE];
}
