//! Transactions

mod hash;

pub use self::hash::{Hash, LENGTH as HASH_LENGTH};
use crate::prelude::*;
use core::{fmt, slice};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use subtle_encoding::base64;
use tendermint_proto::types::Data as RawData;

/// Transactions are arbitrary byte arrays whose contents are validated by the
/// underlying Tendermint application.
#[derive(Clone, Debug, Eq, PartialEq)] // Custom serde serialization used by RPC /broadcast_tx_async endpoint
pub struct Transaction(Vec<u8>);

impl From<Vec<u8>> for Transaction {
    fn from(value: Vec<u8>) -> Self {
        Transaction(value)
    }
}

impl From<Transaction> for Vec<u8> {
    fn from(value: Transaction) -> Self {
        value.0
    }
}

impl Transaction {
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

        Ok(Self::from(bytes))
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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawData", into = "RawData")]
pub struct Data {
    txs: Option<Vec<Transaction>>,
}

impl From<RawData> for Data {
    fn from(value: RawData) -> Self {
        if value.txs.is_empty() {
            return Data::default();
        }
        Data {
            txs: Some(
                value
                    .txs
                    .iter()
                    .map(|tx| Transaction::from(tx.clone()))
                    .collect(),
            ),
        }
    }
}

impl From<Data> for RawData {
    fn from(value: Data) -> Self {
        if value.txs.is_none() {
            return RawData { txs: vec![] };
        }
        RawData {
            txs: value
                .txs
                .unwrap_or_default()
                .iter()
                .map(|tx| tx.clone().into())
                .collect(),
        }
    }
}

impl Data {
    /// Iterate over the transactions in the collection
    pub fn iter(&self) -> slice::Iter<'_, Transaction> {
        self.as_ref().iter()
    }
}

impl AsRef<[Transaction]> for Data {
    fn as_ref(&self) -> &[Transaction] {
        self.txs.as_deref().unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::Transaction;
    use crate::prelude::*;

    #[test]
    fn upper_hex_serialization() {
        let tx = Transaction::from(vec![0xFF, 0x01, 0xFE, 0x02]);
        let tx_hex = format!("{:X}", &tx);
        assert_eq!(&tx_hex, "FF01FE02");
    }
}
