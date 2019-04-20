//! Transactions

use std::slice;
#[cfg(feature = "serde")]
use {
    serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer},
    subtle_encoding::base64,
};

/// Transactions
// TODO(tarcieri): parse transactions (amino?)
#[derive(Clone, Debug)]
pub struct Transaction(Vec<u8>);

impl Transaction {
    /// Create a new raw transaction from a byte vector
    pub fn new<V>(into_vec: V) -> Transaction
    where
        V: Into<Vec<u8>>,
    {
        // TODO(tarcieri): parse/validate transaction contents
        Transaction(into_vec.into())
    }

    /// Serialize this transaction as a bytestring
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = base64::decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(|e| D::Error::custom(format!("{}", e)))?;

        Ok(Transaction::new(bytes))
    }
}

#[cfg(feature = "serde")]
impl Serialize for Transaction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        String::from_utf8(base64::encode(self.to_bytes()))
            .unwrap()
            .serialize(serializer)
    }
}

/// Transaction collections
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, Debug)]
pub struct Collection {
    txs: Option<Vec<Transaction>>,
}

impl Collection {
    /// Create a new evidence collection
    pub fn new<I>(into_evidence: I) -> Collection
    where
        I: Into<Vec<Transaction>>,
    {
        Collection {
            txs: Some(into_evidence.into()),
        }
    }

    /// Convert this collection into a vector
    pub fn into_vec(self) -> Vec<Transaction> {
        self.txs.unwrap_or_else(|| vec![])
    }

    /// Iterate over the transactions in the collection
    pub fn iter(&self) -> slice::Iter<Transaction> {
        self.as_ref().iter()
    }
}

impl AsRef<[Transaction]> for Collection {
    fn as_ref(&self) -> &[Transaction] {
        self.txs
            .as_ref()
            .map(|txs| txs.as_slice())
            .unwrap_or_else(|| &[])
    }
}
