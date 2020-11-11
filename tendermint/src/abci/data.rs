use crate::Error;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use tendermint_proto::serializers::bytes::Base64;

/// ABCI transaction data.
///
/// Transactions are opaque binary blobs which are validated according to
/// application-specific rules.
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
#[serde(try_from = "Base64", into = "Base64")]
pub struct Data(Vec<u8>);

impl TryFrom<Vec<u8>> for Data {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // Todo: Are there any restrictions on the incoming data?
        Ok(Self(value))
    }
}

impl From<Data> for Vec<u8> {
    fn from(value: Data) -> Self {
        value.0
    }
}

impl TryFrom<Base64> for Data {
    type Error = Error;

    fn try_from(value: Base64) -> Result<Self, Self::Error> {
        value.try_into()
    }
}

impl From<Data> for Base64 {
    fn from(value: Data) -> Self {
        Self(value.into())
    }
}

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::abci::Data;
    use std::convert::TryFrom;

    #[test]
    fn test_deserialization() {
        let json = "\"ChYKFGNvbm5lY3Rpb25fb3Blbl9pbml0\"";
        let mydata: Data = serde_json::from_str(json).unwrap();
        assert_eq!(
            mydata.0,
            vec![
                // By chance this is a protobuf struct.
                10, // Field 1 is a String
                22, // Field 1 length is 22
                10, // Sub-field 1 is String
                20, // Sub-field 1 length is 20
                99, 111, 110, 110, 101, 99, 116, 105, 111, 110, 95, 111, 112, 101, 110, 95, 105,
                110, 105, 116 // "connection_open_init"
            ]
        );
    }

    #[test]
    fn test_serialization() {
        let mydata: Data = Data::try_from(vec![1, 2, 3, 4]).unwrap();
        let json = serde_json::to_string(&mydata).unwrap();
        assert_eq!(json, "\"AQIDBA==\"");
    }
}
