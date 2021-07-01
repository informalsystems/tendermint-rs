//! This modules provides type-safe interfaces over the `sled` API,
//! by taking care of (de)serializing keys and values with the
//! CBOR binary encoding.

use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};

use crate::errors::{self as error, Error};
use crate::types::Height;

/// Provides a view over the database for storing key/value pairs at the given prefix.
#[derive(Clone, Debug)]
pub struct HeightIndexedDb<V> {
    tree: sled::Tree,
    marker: PhantomData<V>,
}

impl<V> HeightIndexedDb<V> {
    /// Create a view over the database for storing key/value pairs at the given prefix.
    pub fn new(tree: sled::Tree) -> Self {
        Self {
            tree,
            marker: PhantomData,
        }
    }
}

fn key_bytes(height: Height) -> [u8; 8] {
    // we need to store the height in big-endian form for
    // sled's iterators and ordered operations to work properly.
    // See https://github.com/spacejam/sled#a-note-on-lexicographic-ordering-and-endianness
    height.value().to_be_bytes()
}

impl<V> HeightIndexedDb<V>
where
    V: Serialize + DeserializeOwned,
{
    /// Get the value associated with the given height within this tree
    pub fn get(&self, height: Height) -> Result<Option<V>, Error> {
        let key = key_bytes(height);
        let value = self.tree.get(key).map_err(error::sled_error)?;

        match value {
            Some(bytes) => {
                let value = serde_cbor::from_slice(&bytes).map_err(error::serde_cbor_error)?;
                Ok(value)
            }
            None => Ok(None),
        }
    }

    /// Check whether there exists a value associated with the given height within this tree
    pub fn contains_key(&self, height: Height) -> Result<bool, Error> {
        let key = key_bytes(height);

        let exists = self.tree.contains_key(key).map_err(error::sled_error)?;

        Ok(exists)
    }

    /// Insert a value associated with a height within this tree
    pub fn insert(&self, height: Height, value: &V) -> Result<(), Error> {
        let key = key_bytes(height);
        let bytes = serde_cbor::to_vec(&value).map_err(error::serde_cbor_error)?;

        self.tree.insert(key, bytes).map_err(error::sled_error)?;

        Ok(())
    }

    /// Remove the value associated with a height within this tree
    pub fn remove(&self, height: Height) -> Result<(), Error> {
        let key = key_bytes(height);

        self.tree.remove(key).map_err(error::sled_error)?;

        Ok(())
    }

    /// Return an iterator over all values within this tree
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = V> {
        self.tree
            .iter()
            .flatten()
            .flat_map(|(_, v)| serde_cbor::from_slice(&v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn iter_next_returns_lowest_height() {
        let tmp_dir = tempdir().unwrap();
        let db = sled::open(tmp_dir).unwrap();
        let kv = HeightIndexedDb::new(db.open_tree("light_store/verified").unwrap());

        for i in 1..=1000_u32 {
            kv.insert(i.into(), &i).unwrap();
        }

        for i in (1000..=2000_u32).rev() {
            kv.insert(i.into(), &i).unwrap();
        }

        let mut iter = kv.iter();
        for i in 1..=2000_u32 {
            assert_eq!(iter.next(), Some(i));
        }
    }

    #[test]
    fn iter_next_back_returns_highest_height() {
        let tmp_dir = tempdir().unwrap();
        let db = sled::open(tmp_dir).unwrap();
        let kv = HeightIndexedDb::new(db.open_tree("light_store/verified").unwrap());

        for i in 1..=1000_u32 {
            kv.insert(i.into(), &i).unwrap();
        }

        for i in (1000..=2000_u32).rev() {
            kv.insert(i.into(), &i).unwrap();
        }

        let mut iter = kv.iter();
        for i in (1..=2000_u32).rev() {
            assert_eq!(iter.next_back(), Some(i));
        }
    }
}
