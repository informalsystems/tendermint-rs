//! This modules provides type-safe interfaces over the `sled` API,
//! by taking care of (de)serializing keys and values with the
//! CBOR binary encoding.

use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};

use crate::errors::{Error, ErrorKind};
use crate::types::Height;

/// Provides a view over the database for storing key/value pairs at the given prefix.
#[derive(Clone, Debug)]
pub struct HeightIndexedDb<V> {
    tree: sled::Tree,
    marker: PhantomData<V>,
}

impl<V> HeightIndexedDb<V> {
    /// Create a view over the database for storing key/value pairs within the given `Tree`
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
    /// Get the value associated with a height within this view in the given sled database.
    pub fn get(&self, height: Height) -> Result<Option<V>, Error> {
        let key = key_bytes(height);
        let bytes = self
            .tree
            .get(key)
            .map_err(|e| ErrorKind::Store.context(e))?;

        match bytes {
            Some(bytes) => {
                let value =
                    serde_cbor::from_slice(&bytes).map_err(|e| ErrorKind::Store.context(e))?;
                Ok(value)
            }
            None => Ok(None),
        }
    }

    /// Check whether there exists a height within this view in the given sled database.
    pub fn contains_key(&self, height: Height) -> Result<bool, Error> {
        let key = key_bytes(height);

        let exists = self
            .tree
            .contains_key(key)
            .map_err(|e| ErrorKind::Store.context(e))?;

        Ok(exists)
    }

    /// Insert a value associated with a height within this view in the given sled database.
    pub fn insert(&self, height: Height, value: &V) -> Result<(), Error> {
        let key = key_bytes(height);
        let bytes = serde_cbor::to_vec(&value).map_err(|e| ErrorKind::Store.context(e))?;

        self.tree
            .insert(key, bytes)
            .map_err(|e| ErrorKind::Store.context(e))?;

        Ok(())
    }

    /// Remove the value associated with a key within this view in the given sled database.
    pub fn remove(&self, height: Height) -> Result<(), Error> {
        let key = key_bytes(height);

        self.tree
            .remove(key)
            .map_err(|e| ErrorKind::Store.context(e))?;

        Ok(())
    }

    /// Iterate over all values within this view in the given sled database.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = V> {
        self.tree
            .iter()
            .flatten()
            .map(|(_, v)| serde_cbor::from_slice(&v))
            .flatten()
    }
}

// TODO: The test below is currently disabled because it fails on CI as we don't have
// access to `/tmp`. Need to figure out how to specify a proper temp dir.

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::types::Height;

//     #[test]
//     fn iter_next_back_returns_highest_height() {
//         const DB_PATH: &str = "/tmp/tendermint_light_client_sled_test/";
//         std::fs::remove_dir_all(DB_PATH).unwrap();
//         let db = sled::open(DB_PATH).unwrap();
//         let kv: KeyValueDb<Height, Height> = key_value("light_store/verified");

//         kv.insert(&db, &1, &1).unwrap();
//         kv.insert(&db, &589473798493, &589473798493).unwrap();
//         kv.insert(&db, &12342425, &12342425).unwrap();
//         kv.insert(&db, &4, &4).unwrap();

//         let mut iter = kv.iter(&db);
//         assert_eq!(iter.next_back(), Some(589473798493));
//         assert_eq!(iter.next_back(), Some(12342425));
//         assert_eq!(iter.next_back(), Some(4));
//         assert_eq!(iter.next_back(), Some(1));
//         assert_eq!(iter.next_back(), None);
//     }
// }
