//! Merkle tree used in Tendermint networks

pub mod proof;
use digest::{consts::U32, Digest, FixedOutputReset};
pub use proof::Proof;

/// Size of Merkle root hash
pub use crate::crypto::HASH_SIZE;
use crate::{crypto::Hasher, prelude::*};

/// Hash is the output of the cryptographic digest function
pub type Hash = [u8; HASH_SIZE];

/// Implementation of Merkle tree hashing for Tendermint.
pub trait MerkleHash: Hasher {
    // tmhash({})
    // Pre and post-conditions: the hasher is in the reset state
    // before and after calling this function.
    fn empty_hash(&mut self) -> Hash;

    // tmhash(0x00 || leaf)
    // Pre and post-conditions: the hasher is in the reset state
    // before and after calling this function.
    fn leaf_hash(&mut self, bytes: &[u8]) -> Hash;

    // tmhash(0x01 || left || right)
    // Pre and post-conditions: the hasher is in the reset state
    // before and after calling this function.
    fn inner_hash(&mut self, left: Hash, right: Hash) -> Hash;

    // Implements recursion into subtrees.
    // Pre and post-conditions: the hasher is in the reset state
    // before and after calling this function.
    fn hash_byte_vectors(&mut self, byte_vecs: &[Vec<u8>]) -> Hash {
        let length = byte_vecs.len();
        match length {
            0 => self.empty_hash(),
            1 => self.leaf_hash(&byte_vecs[0]),
            _ => {
                let split = length.next_power_of_two() / 2;
                let left = self.hash_byte_vectors(&byte_vecs[..split]);
                let right = self.hash_byte_vectors(&byte_vecs[split..]);
                self.inner_hash(left, right)
            },
        }
    }
}

/// A  Blanket implementation of MerkleHash for any Digest
impl<H> MerkleHash for H
where
    H: Digest<OutputSize = U32> + FixedOutputReset + Hasher,
{
    fn empty_hash(&mut self) -> Hash {
        // Get the output of an empty digest state.
        let digest = self.finalize_reset();
        <Self as Hasher>::digest(digest)
    }

    fn leaf_hash(&mut self, bytes: &[u8]) -> Hash {
        // Feed the data to the hasher, prepended with 0x00.
        Digest::update(self, [0x00]);
        Digest::update(self, bytes);

        // Finalize the digest, reset the hasher state.
        let digest = self.finalize_reset();
        <Self as Hasher>::digest(digest)
    }

    fn inner_hash(&mut self, left: Hash, right: Hash) -> Hash {
        // Feed the data to the hasher: 0x1, then left and right data.
        Digest::update(self, [0x01]);
        Digest::update(self, left);
        Digest::update(self, right);

        // Finalize the digest, reset the hasher state
        let digest = self.finalize_reset();
        <Self as Hasher>::digest(digest)
    }
}
#[cfg(test)]
mod tests {
    use subtle_encoding::hex;

    use super::*;
    use crate::crypto::Sha256; // TODO: use non-subtle ?

    #[test]
    fn test_rfc6962_empty_tree() {
        let empty_tree_root_hex =
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let empty_tree_root = &hex::decode(empty_tree_root_hex).unwrap();
        let empty_tree: Vec<Vec<u8>> = vec![vec![]; 0];

        let root = Sha256::default().hash_byte_vectors(&empty_tree);
        assert_eq!(empty_tree_root, &root);
    }

    #[test]
    fn test_rfc6962_empty_leaf() {
        let empty_leaf_root_hex =
            "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d";
        let empty_leaf_root = &hex::decode(empty_leaf_root_hex).unwrap();
        let one_empty_leaf: Vec<Vec<u8>> = vec![vec![]; 1];

        let root = Sha256::default().hash_byte_vectors(&one_empty_leaf);
        assert_eq!(empty_leaf_root, &root);
    }

    #[test]
    fn test_rfc6962_leaf() {
        let leaf_root_hex = "395aa064aa4c29f7010acfe3f25db9485bbd4b91897b6ad7ad547639252b4d56";
        let leaf_string = "L123456";

        let leaf_root = &hex::decode(leaf_root_hex).unwrap();
        let leaf_tree: Vec<Vec<u8>> = vec![leaf_string.as_bytes().to_vec(); 1];

        let root = Sha256::default().hash_byte_vectors(&leaf_tree);
        assert_eq!(leaf_root, &root);
    }

    #[test]
    fn test_rfc6962_tree_of_2() {
        let node_hash_hex = "dc9a0536ff2e196d5a628a5bf377ab247bbddf83342be39699461c1e766e6646";
        let left = b"N123".to_vec();
        let right = b"N456".to_vec();

        let node_hash = &hex::decode(node_hash_hex).unwrap();
        let hash = Sha256::default().hash_byte_vectors(&[left, right]);
        assert_eq!(node_hash, &hash);
    }
}
