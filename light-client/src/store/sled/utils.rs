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
    prefix: Vec<u8>,
    marker: PhantomData<V>,
}

impl<V> HeightIndexedDb<V> {
    /// Create a view over the database for storing key/value pairs at the given prefix.
    pub fn new(prefix: impl Into<Vec<u8>>) -> Self {
        Self {
            prefix: prefix.into(),
            marker: PhantomData,
        }
    }
}

impl<V> HeightIndexedDb<V>
where
    V: Serialize + DeserializeOwned,
{
    fn prefixed_key(&self, height: Height) -> Vec<u8> {
        // we need to store the height in big-endian form for
        // sled's iterators and ordered operations to work properly.
        // See https://github.com/spacejam/sled#a-note-on-lexicographic-ordering-and-endianness
        let key = height.value().to_be_bytes();

        let mut prefix_bytes = self.prefix.clone();
        prefix_bytes.extend(&key);
        prefix_bytes
    }

    /// Get the value associated with a height within this view in the given sled database.
    pub fn get(&self, db: &sled::Db, height: Height) -> Result<Option<V>, Error> {
        let key_bytes = self.prefixed_key(height);
        let value_bytes = db.get(key_bytes).map_err(|e| ErrorKind::Store.context(e))?;

        match value_bytes {
            Some(bytes) => {
                let value =
                    serde_cbor::from_slice(&bytes).map_err(|e| ErrorKind::Store.context(e))?;
                Ok(value)
            }
            None => Ok(None),
        }
    }

    /// Check whether there exists a height within this view in the given sled database.
    pub fn contains_key(&self, db: &sled::Db, height: Height) -> Result<bool, Error> {
        let key_bytes = self.prefixed_key(height);

        let exists = db
            .contains_key(key_bytes)
            .map_err(|e| ErrorKind::Store.context(e))?;

        Ok(exists)
    }

    /// Insert a value associated with a height within this view in the given sled database.
    pub fn insert(&self, db: &sled::Db, height: Height, value: &V) -> Result<(), Error> {
        let key_bytes = self.prefixed_key(height);
        let value_bytes = serde_cbor::to_vec(&value).map_err(|e| ErrorKind::Store.context(e))?;

        db.insert(key_bytes, value_bytes)
            .map_err(|e| ErrorKind::Store.context(e))?;

        Ok(())
    }

    /// Remove the value associated with a key within this view in the given sled database.
    pub fn remove(&self, db: &sled::Db, height: Height) -> Result<(), Error> {
        let key_bytes = self.prefixed_key(height);

        db.remove(key_bytes)
            .map_err(|e| ErrorKind::Store.context(e))?;

        Ok(())
    }

    /// Iterate over all values within this view in the given sled database.
    pub fn iter(&self, db: &sled::Db) -> impl DoubleEndedIterator<Item = V> {
        db.iter()
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
