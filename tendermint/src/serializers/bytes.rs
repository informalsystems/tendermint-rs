//! Serialize/deserialize bytes (Vec<u8>) type
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;
use subtle_encoding::{base64, hex};

/// ByteStringType defines the options what an incoming string can represent.
enum ByteStringType {
    Hex,
    Base64,
    Regular,
}

/// The Visitor struct to decode the incoming string.
struct BytesVisitor {
    string_type: ByteStringType,
}

/// The Visitor implementation
impl<'de> Visitor<'de> for BytesVisitor {
    type Value = Vec<u8>;

    /// Description of expected input
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.string_type {
            ByteStringType::Hex => {
                formatter.write_str("Hex-encoded byte-array in a String or null")
            }
            ByteStringType::Base64 => {
                formatter.write_str("Base64-encoded byte-array in a String or null")
            }
            ByteStringType::Regular => formatter.write_str("Byte-array in a String or null"),
        }
    }

    /// If incoming is 'null', return an empty array.
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(vec![])
    }

    /// Decode the incoming string based on what string type it is.
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;

        match self.string_type {
            ByteStringType::Hex => hex::decode_upper(&string)
                .or_else(|_| hex::decode(&string))
                .map_err(Error::custom),
            ByteStringType::Base64 => base64::decode(&string).map_err(Error::custom),
            ByteStringType::Regular => Ok(string.as_bytes().to_vec()),
        }
    }
}

/// Serialize into hexstring, deserialize from hexstring
pub mod hexstring {
    use serde::{ser::Error, Deserializer, Serializer};
    use subtle_encoding::hex;

    /// Deserialize hexstring into Vec<u8>
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(super::BytesVisitor {
            string_type: super::ByteStringType::Hex,
        })
    }

    /// Serialize from T into hexstring
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        let hex_bytes = hex::encode(value.as_ref());
        let hex_string = String::from_utf8(hex_bytes).map_err(Error::custom)?;
        serializer.serialize_str(&hex_string)
    }
}

/// Serialize into base64string, deserialize from base64string
pub mod base64string {
    use serde::{ser::Error, Deserializer, Serializer};
    use subtle_encoding::base64;

    /// Deserialize base64string into Vec<u8>
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(super::BytesVisitor {
            string_type: super::ByteStringType::Base64,
        })
    }

    /// Serialize from T into base64string
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        let base64_bytes = base64::encode(value.as_ref());
        let base64_string = String::from_utf8(base64_bytes).map_err(Error::custom)?;
        serializer.serialize_str(&base64_string)
    }
}

/// Serialize into string, deserialize from string
pub mod string {
    use serde::{ser::Error, Deserializer, Serializer};

    /// Deserialize string into Vec<u8>
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(super::BytesVisitor {
            string_type: super::ByteStringType::Regular,
        })
    }

    /// Serialize from T into string
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        let string = String::from_utf8(value.as_ref().to_vec()).map_err(Error::custom)?;
        serializer.serialize_str(&string)
    }
}
