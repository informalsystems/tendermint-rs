//! Merkle tree used in Tendermint networks

use sha2::{Digest, Sha256};

/// Size of Merkle root hash
pub const HASH_SIZE: usize = 32;

/// Compute a simple Merkle root from the arbitrary sized byte slices
pub fn simple_hash_from_byte_slices(byte_slices: &[&[u8]]) -> [u8; HASH_SIZE] {
    let length = byte_slices.len();
    match length {
        0 => [0; HASH_SIZE],
        1 => leaf_hash(byte_slices[0]),
        _ => {
            let k = get_split_point(length);
            let left = simple_hash_from_byte_slices(&byte_slices[..k]);
            let right = simple_hash_from_byte_slices(&byte_slices[k..]);
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

// tmhash(0x00 || leaf)
fn leaf_hash(bytes: &[u8]) -> [u8; HASH_SIZE] {
    // make a new array starting with 0 and copy in the bytes
    let mut leaf_bytes = [0u8; HASH_SIZE + 1];
    leaf_bytes[1..].copy_from_slice(bytes);

    // hash it !
    let digest = Sha256::digest(&leaf_bytes);

    // copy the GenericArray out
    let mut hash_bytes = [0u8; HASH_SIZE];
    hash_bytes.copy_from_slice(&digest);
    hash_bytes
}

// tmhash(0x01 || left || right)
fn inner_hash(left: &[u8], right: &[u8]) -> [u8; HASH_SIZE] {
    // make a new array starting with 0x1 and copy in the bytes
    let mut inner_bytes = [0u8; HASH_SIZE * 2 + 1];
    inner_bytes[0] = 0x01;
    inner_bytes[1..].copy_from_slice(left);
    inner_bytes[HASH_SIZE + 1..].copy_from_slice(right);

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
    fn test_empty_leaf() {}
}
