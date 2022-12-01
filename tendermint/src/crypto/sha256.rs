use digest::{consts::U32, Digest, FixedOutputReset};

/// Length of a SHA256 hash in bytes.
pub const HASH_SIZE: usize = 32;

/// A SHA256 digest implementation.
///
/// This trait is blanket-implemented, and it puts a more user-friendly face
/// over the APIs of the `digest` framework.
pub trait Sha256: Send + Sync {
    fn new() -> Self;
    fn update(&mut self, data: impl AsRef<[u8]>);
    fn finalize(self) -> [u8; HASH_SIZE];
    fn finalize_reset(&mut self) -> [u8; HASH_SIZE];
}

impl<H> Sha256 for H
where
    H: Digest<OutputSize = U32> + FixedOutputReset + Send + Sync,
{
    fn new() -> Self {
        Digest::new()
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        Digest::update(self, data)
    }

    fn finalize(self) -> [u8; HASH_SIZE] {
        let digest = Digest::finalize(self);
        // copy the GenericArray out
        let mut hash = [0u8; HASH_SIZE];
        hash.copy_from_slice(&digest);
        hash
    }

    fn finalize_reset(&mut self) -> [u8; HASH_SIZE] {
        let digest = Digest::finalize_reset(self);
        // copy the GenericArray out
        let mut hash = [0u8; HASH_SIZE];
        hash.copy_from_slice(&digest);
        hash
    }
}
