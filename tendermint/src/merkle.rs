//! Merkle tree used in Tendermint networks

pub mod proof;

use crate::prelude::*;
use sha2::{Digest, Sha256};

/// Size of Merkle root hash
pub const HASH_SIZE: usize = 32;

/// Hash is the output of the cryptographic digest function
pub type Hash = [u8; HASH_SIZE];

/// Compute a simple Merkle root from vectors of arbitrary byte vectors.
/// The leaves of the tree are the bytes of the given byte vectors in
/// the given order.
pub fn simple_hash_from_byte_vectors(byte_vecs: Vec<Vec<u8>>) -> Hash {
    simple_hash_from_byte_slices_inner(byte_vecs.as_slice())
}

// recurse into subtrees
fn simple_hash_from_byte_slices_inner(byte_slices: &[Vec<u8>]) -> Hash {
    let length = byte_slices.len();
    match length {
        0 => empty_hash(),
        1 => leaf_hash(byte_slices[0].as_slice()),
        _ => {
            let k = get_split_point(length);
            let left = simple_hash_from_byte_slices_inner(&byte_slices[..k]);
            let right = simple_hash_from_byte_slices_inner(&byte_slices[k..]);
            inner_hash(&left, &right)
        }
    }
}

// returns the largest power of 2 less than length
fn get_split_point(length: usize) -> usize {
    match length {
        0 => panic!("tree is empty!"),
        1 => panic!("tree has only one element!"),
        2 => 1,
        _ => length.next_power_of_two() / 2,
    }
}

// tmhash({})
fn empty_hash() -> Hash {
    // the empty string / byte slice
    let empty = Vec::with_capacity(0);

    // hash it !
    let digest = Sha256::digest(&empty);

    // copy the GenericArray out
    let mut hash_bytes = [0u8; HASH_SIZE];
    hash_bytes.copy_from_slice(&digest);
    hash_bytes
}

// tmhash(0x00 || leaf)
fn leaf_hash(bytes: &[u8]) -> Hash {
    // make a new array starting with 0 and copy in the bytes
    let mut leaf_bytes = Vec::with_capacity(bytes.len() + 1);
    leaf_bytes.push(0x00);
    leaf_bytes.extend_from_slice(bytes);

    // hash it !
    let digest = Sha256::digest(&leaf_bytes);

    // copy the GenericArray out
    let mut hash_bytes = [0u8; HASH_SIZE];
    hash_bytes.copy_from_slice(&digest);
    hash_bytes
}

// tmhash(0x01 || left || right)
fn inner_hash(left: &[u8], right: &[u8]) -> Hash {
    // make a new array starting with 0x1 and copy in the bytes
    let mut inner_bytes = Vec::with_capacity(left.len() + right.len() + 1);
    inner_bytes.push(0x01);
    inner_bytes.extend_from_slice(left);
    inner_bytes.extend_from_slice(right);

    // hash it !
    let digest = Sha256::digest(&inner_bytes);

    // copy the GenericArray out
    let mut hash_bytes = [0u8; HASH_SIZE];
    hash_bytes.copy_from_slice(&digest);
    hash_bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use subtle_encoding::hex; // TODO: use non-subtle ?

    #[test]
    fn test_get_split_point() {
        assert_eq!(get_split_point(2), 1);
        assert_eq!(get_split_point(3), 2);
        assert_eq!(get_split_point(4), 2);
        assert_eq!(get_split_point(5), 4);
        assert_eq!(get_split_point(10), 8);
        assert_eq!(get_split_point(20), 16);
        assert_eq!(get_split_point(100), 64);
        assert_eq!(get_split_point(255), 128);
        assert_eq!(get_split_point(256), 128);
        assert_eq!(get_split_point(257), 256);
    }

    #[test]
    fn test_rfc6962_empty_tree() {
        let empty_tree_root_hex =
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let empty_tree_root = &hex::decode(empty_tree_root_hex).unwrap();
        let empty_tree: Vec<Vec<u8>> = vec![vec![]; 0];

        let root = simple_hash_from_byte_vectors(empty_tree);
        assert_eq!(empty_tree_root, &root);
    }

    #[test]
    fn test_rfc6962_empty_leaf() {
        let empty_leaf_root_hex =
            "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d";
        let empty_leaf_root = &hex::decode(empty_leaf_root_hex).unwrap();
        let one_empty_leaf: Vec<Vec<u8>> = vec![vec![]; 1];

        let root = simple_hash_from_byte_vectors(one_empty_leaf);
        assert_eq!(empty_leaf_root, &root);
    }

    #[test]
    fn test_rfc6962_leaf() {
        let leaf_root_hex = "395aa064aa4c29f7010acfe3f25db9485bbd4b91897b6ad7ad547639252b4d56";
        let leaf_string = "L123456";

        let leaf_root = &hex::decode(leaf_root_hex).unwrap();
        let leaf_tree: Vec<Vec<u8>> = vec![leaf_string.as_bytes().to_vec(); 1];

        let root = simple_hash_from_byte_vectors(leaf_tree);
        assert_eq!(leaf_root, &root);
    }

    #[test]
    fn test_rfc6962_node() {
        let node_hash_hex = "aa217fe888e47007fa15edab33c2b492a722cb106c64667fc2b044444de66bbb";
        let left_string = "N123";
        let right_string = "N456";

        let node_hash = &hex::decode(node_hash_hex).unwrap();
        let hash = inner_hash(left_string.as_bytes(), right_string.as_bytes());
        assert_eq!(node_hash, &hash);
    }
}
