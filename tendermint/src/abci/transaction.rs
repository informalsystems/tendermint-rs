//! Transactions

mod hash;

pub use self::hash::Hash;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, slice};
use subtle_encoding::base64;

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
        self.0
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

impl fmt::UpperHex for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.as_bytes() {
            write!(f, "{:02X}", byte)?;
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = base64::decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(|e| D::Error::custom(format!("{}", e)))?;

        Ok(Self::new(bytes))
    }
}

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
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#data>
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
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
        self.txs.unwrap_or_default()
    }

    /// Iterate over the transactions in the collection
    pub fn iter(&self) -> slice::Iter<'_, Transaction> {
        self.as_ref().iter()
    }
}

impl AsRef<[Transaction]> for Data {
    fn as_ref(&self) -> &[Transaction] {
        self.txs.as_deref().unwrap_or_else(|| &[])
    }
}

#[cfg(test)]
mod tests {
    use super::Transaction;

    #[test]
    fn upper_hex_serialization() {
        let tx = Transaction::new(vec![0xFF, 0x01, 0xFE, 0x02]);
        let tx_hex = format!("{:X}", &tx);
        assert_eq!(&tx_hex, "FF01FE02");
    }
}
