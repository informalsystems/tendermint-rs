//! This modules provides type-safe interfaces over the `sled` API,
//! by taking care of (de)serializing keys and values with the
//! CBOR binary encoding.

use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

use crate::errors::{Error, ErrorKind};

/// Provides a view over the database for storing a single value at the given prefix.
pub fn single<V>(prefix: impl Into<Vec<u8>>) -> SingleDb<V> {
    SingleDb::new(prefix)
}

/// Provides a view over the database for storing key/value pairs at the given prefix.
pub fn key_value<K, V>(prefix: impl Into<Vec<u8>>) -> KeyValueDb<K, V> {
    KeyValueDb::new(prefix)
}

/// Provides a view over the database for storing a single value at the given prefix.
pub struct SingleDb<V>(KeyValueDb<(), V>);

impl<V> SingleDb<V> {
    pub fn new(prefix: impl Into<Vec<u8>>) -> Self {
        Self(KeyValueDb::new(prefix))
    }
}

impl<V> SingleDb<V>
where
    V: Serialize + DeserializeOwned,
{
    pub fn get(&self, db: &sled::Db) -> Result<Option<V>, Error> {
        self.0.get(&db, &())
    }

    pub fn set(&self, db: &sled::Db, value: &V) -> Result<(), Error> {
        self.0.insert(&db, &(), &value)
    }
}

/// Provides a view over the database for storing key/value pairs at the given prefix.
#[derive(Clone, Debug)]
pub struct KeyValueDb<K, V> {
    prefix: Vec<u8>,
    marker: PhantomData<(K, V)>,
}

impl<K, V> KeyValueDb<K, V> {
    pub fn new(prefix: impl Into<Vec<u8>>) -> Self {
        Self {
            prefix: prefix.into(),
            marker: PhantomData,
        }
    }
}

impl<K, V> KeyValueDb<K, V>
where
    K: Serialize,
    V: Serialize + DeserializeOwned,
{
    fn prefixed_key(&self, mut key_bytes: Vec<u8>) -> Vec<u8> {
        let mut prefix_bytes = self.prefix.clone();
        prefix_bytes.append(&mut key_bytes);
        prefix_bytes
    }

    pub fn get(&self, db: &sled::Db, key: &K) -> Result<Option<V>, Error> {
        let key_bytes = serde_cbor::to_vec(&key).map_err(|e| ErrorKind::Store.context(e))?;
        let prefixed_key_bytes = self.prefixed_key(key_bytes);

        let value_bytes = db
            .get(prefixed_key_bytes)
            .map_err(|e| ErrorKind::Store.context(e))?;

        match value_bytes {
            Some(bytes) => {
                let value =
                    serde_cbor::from_slice(&bytes).map_err(|e| ErrorKind::Store.context(e))?;
                Ok(value)
            }
            None => Ok(None),
        }
    }

    pub fn contains_key(&self, db: &sled::Db, key: &K) -> Result<bool, Error> {
        let key_bytes = serde_cbor::to_vec(&key).map_err(|e| ErrorKind::Store.context(e))?;
        let prefixed_key_bytes = self.prefixed_key(key_bytes);

        let exists = db
            .contains_key(prefixed_key_bytes)
            .map_err(|e| ErrorKind::Store.context(e))?;

        Ok(exists)
    }

    pub fn insert(&self, db: &sled::Db, key: &K, value: &V) -> Result<(), Error> {
        let key_bytes = serde_cbor::to_vec(&key).map_err(|e| ErrorKind::Store.context(e))?;
        let prefixed_key_bytes = self.prefixed_key(key_bytes);
        let value_bytes = serde_cbor::to_vec(&value).map_err(|e| ErrorKind::Store.context(e))?;

        db.insert(prefixed_key_bytes, value_bytes)
            .map(|_| ())
            .map_err(|e| ErrorKind::Store.context(e))?;

        Ok(())
    }

    pub fn remove(&self, db: &sled::Db, key: &K) -> Result<(), Error> {
        let key_bytes = serde_cbor::to_vec(&key).map_err(|e| ErrorKind::Store.context(e))?;
        let prefixed_key_bytes = self.prefixed_key(key_bytes);

        db.remove(prefixed_key_bytes)
            .map_err(|e| ErrorKind::Store.context(e))?;

        Ok(())
    }

    pub fn iter(&self, db: &sled::Db) -> impl Iterator<Item = V> {
        db.iter()
            .flatten()
            .map(|(_, v)| serde_cbor::from_slice(&v))
            .flatten()
    }
}
