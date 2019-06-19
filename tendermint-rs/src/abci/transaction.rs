//! Transactions

mod hash;

pub use self::hash::Hash;
use std::slice;
#[cfg(feature = "serde")]
use {
    serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer},
    subtle_encoding::base64,
};

/// Transactions are arbitrary byte arrays whose contents are validated by the
/// underlying Tendermint application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction(Vec<u8>);

impl Transaction {
    /// Create a new raw transaction from a byte vector
    pub fn new<V>(into_vec: V) -> Transaction
    where
        V: Into<Vec<u8>>,
    {
        Transaction(into_vec.into())
    }

    /// Convert this transaction into a byte vector
    pub fn into_vec(self) -> Vec<u8> {
        self.0.clone()
    }

    /// Borrow the contents of this transaction as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl AsRef<[u8]> for Transaction {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = base64::decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(|e| D::Error::custom(format!("{}", e)))?;

        Ok(Self::new(bytes))
    }
}

#[cfg(feature = "serde")]
impl Serialize for Transaction {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        String::from_utf8(base64::encode(self.as_bytes()))
            .unwrap()
            .serialize(serializer)
    }
}

/// Transaction data is a wrapper for a list of transactions, where
/// transactions are arbitrary byte arrays.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#data>
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, Debug)]
pub struct Data {
    txs: Option<Vec<Transaction>>,
}

impl Data {
    /// Create a new transaction data collection
    pub fn new<I>(into_transactions: I) -> Data
    where
        I: Into<Vec<Transaction>>,
    {
        Data {
            txs: Some(into_transactions.into()),
        }
    }

    /// Convert this collection into a vector
    pub fn into_vec(self) -> Vec<Transaction> {
        self.iter().cloned().collect()
    }

    /// Iterate over the transactions in the collection
    pub fn iter(&self) -> slice::Iter<Transaction> {
        self.as_ref().iter()
    }
}

impl AsRef<[Transaction]> for Data {
    fn as_ref(&self) -> &[Transaction] {
        self.txs.as_ref().map(Vec::as_slice).unwrap_or_else(|| &[])
    }
}
