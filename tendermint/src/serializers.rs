//! Serde serializers
//!
//! This example shows how to serialize Vec<u8> into different types of strings:
//! #[derive(Serialize, Deserialize)]
//! struct ByteTypes {
//!
//!     #[serde(with="serializers::bytes::hexstring")]
//!     hexbytes: Vec<u8>,
//!
//!     #[serde(with="serializers::bytes::base64string")]
//!     base64bytes: Vec<u8>,
//!
//!     #[serde(with="serializers::bytes::string")]
//!     bytes: Vec<u8>,
//!
//! }
//!
//! Available serializers:
//! i64                  <-> string:               #[serde(with="serializers::primitives::string")]
//! u64                  <-> string:               #[serde(with="serializers::primitives::string")]
//! std::time::Dureation <-> nanoseconds as string #[serde(with="serializers::timeduration::string")]
//! Vec<u8>              <-> HexString:            #[serde(with="serializers::bytes::hexstring")]
//! Vec<u8>              <-> Base64String:         #[serde(with="serializers::bytes::base64string")]
//! Vec<u8>              <-> String:               #[serde(with="serializers::bytes::string")]
//!
//! Any type that has the "FromStr" trait can be serialized into a string with serializers::primitives::string.
//!

use crate::account::{Id, LENGTH};
use crate::{block, Hash, Signature};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// Serialize/deserialize primitive types (i64, u64, etc)
pub mod primitives {

    /// Serialize into string, deserialize from string
    pub mod string {
        use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

        /// Deserialize string into T
        pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
        where
            D: Deserializer<'de>,
            T: std::str::FromStr,
            <T as std::str::FromStr>::Err: std::fmt::Display,
        {
            String::deserialize(deserializer)?
                .parse::<T>()
                .map_err(|e| D::Error::custom(format!("{}", e)))
        }

        /// Serialize from T into string
        pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
            T: std::fmt::Display,
        {
            format!("{}", value).serialize(serializer)
        }
    }
}

/// Serialize/deserialize std::time::Duration type
pub mod timeduration {

    /// Serialize into string, deserialize from string
    pub mod string {
        use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
        use std::time::Duration;

        /// Deserialize string into Duration
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
        where
            D: Deserializer<'de>,
        {
            let value = String::deserialize(deserializer)?
                .parse::<u64>()
                .map_err(|e| D::Error::custom(format!("{}", e)))?;

            Ok(Duration::from_nanos(value))
        }

        /// Serialize from Duration into string
        pub fn serialize<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            format!("{}", value.as_nanos()).serialize(serializer)
        }
    }
}

/// Serialize/deserialize bytes (Vec<u8>) type
pub mod bytes {

    /// Serialize into hexstring, deserialize from hexstring
    pub mod hexstring {
        use serde::{
            de::Error as deError, ser::Error as serError, Deserialize, Deserializer, Serializer,
        };
        use subtle_encoding::hex;

        /// Deserialize hexstring into Vec<u8>
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let string = String::deserialize(deserializer)?;

            hex::decode_upper(&string)
                .or_else(|_| hex::decode(&string))
                .map_err(deError::custom)
        }

        /// Serialize from T into hexstring
        pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
            T: AsRef<[u8]>,
        {
            let hex_bytes = hex::encode(value.as_ref());
            let hex_string = String::from_utf8(hex_bytes).map_err(serError::custom)?;
            serializer.serialize_str(&hex_string)
        }
    }

    /// Serialize into base64string, deserialize from base64string
    pub mod base64string {
        use serde::{
            de::Error as deError, ser::Error as serError, Deserialize, Deserializer, Serializer,
        };
        use subtle_encoding::base64;

        /// Deserialize base64string into Vec<u8>
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let string = String::deserialize(deserializer)?;

            base64::decode(&string).map_err(deError::custom)
        }

        /// Serialize from T into base64string
        pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
            T: AsRef<[u8]>,
        {
            let base64_bytes = base64::encode(value.as_ref());
            let base64_string = String::from_utf8(base64_bytes).map_err(serError::custom)?;
            serializer.serialize_str(&base64_string)
        }
    }

    /// Serialize into string, deserialize from string
    pub mod string {
        use serde::{
            de::Error as _, ser::Error as serError, Deserialize, Deserializer, Serializer,
        };

        /// Deserialize string into Vec<u8>
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
        where
            D: Deserializer<'de>,
        {
            String::deserialize(deserializer)
                .map(|m| m.as_bytes().to_vec())
                .map_err(|e| D::Error::custom(format!("{}", e)))
        }

        /// Serialize from T into string
        pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
            T: AsRef<[u8]>,
        {
            let string = String::from_utf8(value.as_ref().to_vec()).map_err(serError::custom)?;
            serializer.serialize_str(&string)
        }
    }
}

// Todo: Refactor the "Option"-based serializers below.
//  Most of them are not needed if the structs are defined well (with enums).

pub(crate) fn parse_non_empty_hash<'de, D>(deserializer: D) -> Result<Option<Hash>, D::Error>
where
    D: Deserializer<'de>,
{
    let o: Option<String> = Option::deserialize(deserializer)?;
    match o.filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(s) => Ok(Some(
            Hash::from_str(&s).map_err(|err| D::Error::custom(format!("{}", err)))?,
        )),
    }
}

#[allow(dead_code)]
pub(crate) fn serialize_option_base64<S>(
    maybe_bytes: &Option<Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    struct Wrapper<'a>(#[serde(with = "bytes::base64string")] &'a Vec<u8>);

    match maybe_bytes {
        Some(bytes) => Wrapper(bytes).serialize(serializer),
        None => maybe_bytes.serialize(serializer),
    }
}

#[allow(dead_code)]
pub(crate) fn parse_option_base64<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(with = "bytes::base64string")] Vec<u8>);

    let v = Option::deserialize(deserializer)?;
    Ok(v.map(|Wrapper(a)| a))
}

/// Parse empty block id as None.
pub(crate) fn parse_non_empty_block_id<'de, D>(
    deserializer: D,
) -> Result<Option<block::Id>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Parts {
        #[serde(with = "primitives::string")]
        total: u64,
        hash: String,
    }
    #[derive(Deserialize)]
    struct BlockId {
        hash: String,
        parts: Parts,
    }
    let tmp_id = BlockId::deserialize(deserializer)?;
    if tmp_id.hash.is_empty() {
        Ok(None)
    } else {
        Ok(Some(block::Id {
            hash: Hash::from_str(&tmp_id.hash)
                .map_err(|err| D::Error::custom(format!("{}", err)))?,
            parts: if tmp_id.parts.hash.is_empty() {
                None
            } else {
                Some(block::parts::Header {
                    total: tmp_id.parts.total,
                    hash: Hash::from_str(&tmp_id.parts.hash)
                        .map_err(|err| D::Error::custom(format!("{}", err)))?,
                })
            },
        }))
    }
}

pub(crate) fn parse_non_empty_id<'de, D>(deserializer: D) -> Result<Option<Id>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        // TODO: how can we avoid rewriting code here?
        match Id::from_str(&s).map_err(|_| {
            D::Error::custom(format!(
                "expected {}-character hex string, got {:?}",
                LENGTH * 2,
                s
            ))
        }) {
            Ok(id) => Ok(Option::from(id)),
            Err(_) => Ok(None),
        }
    }
}

pub(crate) fn parse_non_empty_signature<'de, D>(
    deserializer: D,
) -> Result<Option<Signature>, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(|x: Option<_>| x.unwrap_or(None))
}
