//! Merkle tree used in Tendermint networks

pub mod proof;

pub use proof::Proof;

use digest::{consts::U32, Digest, FixedOutputReset};

use crate::prelude::*;

/// Size of Merkle root hash
pub const HASH_SIZE: usize = 32;

/// Hash is the output of the cryptographic digest function
pub type Hash = [u8; HASH_SIZE];

/// Compute a simple Merkle root from vectors of arbitrary byte vectors.
/// The leaves of the tree are the bytes of the given byte vectors in
/// the given order.
pub fn simple_hash_from_byte_vectors<H: Digest<OutputSize = U32> + FixedOutputReset>(
    byte_vecs: &[Vec<u8>],
) -> Hash {
    let mut hasher = H::new();
    simple_hash_from_byte_vectors_inner(&mut hasher, byte_vecs)
}

// Recurse into subtrees.
// Pre and post-conditions: the hasher is in the reset state before and after calling this function.
fn simple_hash_from_byte_vectors_inner<H: Digest<OutputSize = U32> + FixedOutputReset>(
    hasher: &mut H,
    byte_vecs: &[Vec<u8>],
) -> Hash {
    let length = byte_vecs.len();
    match length {
        0 => empty_hash(hasher),
        1 => leaf_hash(hasher, &byte_vecs[0]),
        _ => {
            let split = length.next_power_of_two() / 2;
            let left = simple_hash_from_byte_vectors_inner(hasher, &byte_vecs[..split]);
            let right = simple_hash_from_byte_vectors_inner(hasher, &byte_vecs[split..]);
            inner_hash(hasher, &left, &right)
        },
    }
}

// tmhash({})
// Pre and post-conditions: the hasher is in the reset state before and after calling this function.
fn empty_hash<H: Digest<OutputSize = U32> + FixedOutputReset>(hasher: &mut H) -> Hash {
    // Get the hash of an empty digest state
    let digest = hasher.finalize_reset();

    // copy the GenericArray out
    let mut hash_bytes = [0u8; HASH_SIZE];
    hash_bytes.copy_from_slice(&digest);
    hash_bytes
}

// tmhash(0x00 || leaf)
// Pre and post-conditions: the hasher is in the reset state before and after calling this function.
fn leaf_hash<H: Digest<OutputSize = U32> + FixedOutputReset>(hasher: &mut H, bytes: &[u8]) -> Hash {
    // Feed the data to the hasher, prepended with 0x00
    Digest::update(hasher, &[0x00]);
    Digest::update(hasher, bytes);

    // Finalize the digest, reset the hasher state
    let digest = hasher.finalize_reset();

    // copy the GenericArray out
    let mut hash_bytes = [0u8; HASH_SIZE];
    hash_bytes.copy_from_slice(&digest);
    hash_bytes
}

// tmhash(0x01 || left || right)
// Pre and post-conditions: the hasher is in the reset state before and after calling this function.
fn inner_hash<H: Digest<OutputSize = U32> + FixedOutputReset>(
    hasher: &mut H,
    left: &[u8],
    right: &[u8],
) -> Hash {
    // Feed the data to the hasher 0x1, then left and right data.
    Digest::update(hasher, &[0x01]);
    Digest::update(hasher, left);
    Digest::update(hasher, right);

    // Finalize the digest, reset the hasher state
    let digest = hasher.finalize_reset();

    // copy the GenericArray out
    let mut hash_bytes = [0u8; HASH_SIZE];
    hash_bytes.copy_from_slice(&digest);
    hash_bytes
}

#[cfg(test)]
mod tests {
    use sha2::Sha256;
    use subtle_encoding::hex;

    use super::*; // TODO: use non-subtle ?

    #[test]
    fn test_rfc6962_empty_tree() {
        let empty_tree_root_hex =
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let empty_tree_root = &hex::decode(empty_tree_root_hex).unwrap();
        let empty_tree: Vec<Vec<u8>> = vec![vec![]; 0];

        let root = simple_hash_from_byte_vectors::<Sha256>(&empty_tree);
        assert_eq!(empty_tree_root, &root);
    }

    #[test]
    fn test_rfc6962_empty_leaf() {
        let empty_leaf_root_hex =
            "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d";
        let empty_leaf_root = &hex::decode(empty_leaf_root_hex).unwrap();
        let one_empty_leaf: Vec<Vec<u8>> = vec![vec![]; 1];

        let root = simple_hash_from_byte_vectors::<Sha256>(&one_empty_leaf);
        assert_eq!(empty_leaf_root, &root);
    }

    #[test]
    fn test_rfc6962_leaf() {
        let leaf_root_hex = "395aa064aa4c29f7010acfe3f25db9485bbd4b91897b6ad7ad547639252b4d56";
        let leaf_string = "L123456";

        let leaf_root = &hex::decode(leaf_root_hex).unwrap();
        let leaf_tree: Vec<Vec<u8>> = vec![leaf_string.as_bytes().to_vec(); 1];

        let root = simple_hash_from_byte_vectors::<Sha256>(&leaf_tree);
        assert_eq!(leaf_root, &root);
    }

    #[test]
    fn test_rfc6962_node() {
        let node_hash_hex = "aa217fe888e47007fa15edab33c2b492a722cb106c64667fc2b044444de66bbb";
        let left_string = "N123";
        let right_string = "N456";

        let node_hash = &hex::decode(node_hash_hex).unwrap();
        let mut hasher = Sha256::new();
        let hash = inner_hash(&mut hasher, left_string.as_bytes(), right_string.as_bytes());
        assert_eq!(node_hash, &hash);
    }

    #[test]
    fn test_rfc6962_tree_of_2() {
        let node_hash_hex = "dc9a0536ff2e196d5a628a5bf377ab247bbddf83342be39699461c1e766e6646";
        let left = b"N123".to_vec();
        let right = b"N456".to_vec();

        let node_hash = &hex::decode(node_hash_hex).unwrap();
        let hash = simple_hash_from_byte_vectors::<Sha256>(&[left, right]);
        assert_eq!(node_hash, &hash);
    }
}
