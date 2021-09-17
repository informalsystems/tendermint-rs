use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// ABCI transaction data.
///
/// Transactions are opaque binary blobs which are validated according to
/// application-specific rules.
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Data(#[serde(with = "crate::serializers::bytes::base64string")] Vec<u8>);

impl From<Vec<u8>> for Data {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<Data> for Vec<u8> {
    fn from(value: Data) -> Self {
        value.0
    }
}

impl Data {
    /// Get value
    pub fn value(&self) -> &Vec<u8> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::abci::Data;
    use crate::prelude::*;

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
        let mydata: Data = vec![1, 2, 3, 4].into();
        let json = serde_json::to_string(&mydata).unwrap();
        assert_eq!(json, "\"AQIDBA==\"");
    }
}
