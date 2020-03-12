//! Serde serializers

use crate::account::{Id, LENGTH};
use crate::{block, Hash, Signature};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use std::time::Duration;
use subtle_encoding::{base64, hex};

/// Parse `i64` from a JSON string
pub(crate) fn parse_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse::<i64>()
        .map_err(|e| D::Error::custom(format!("{}", e)))
}

/// Serialize `i64` as a JSON string
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn serialize_i64<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    format!("{}", value).serialize(serializer)
}

/// Parse `u64` from a JSON string
pub(crate) fn parse_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse::<u64>()
        .map_err(|e| D::Error::custom(format!("{}", e)))
}

/// Serialize `u64` as a JSON string
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn serialize_u64<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    format!("{}", value).serialize(serializer)
}

/// Parse `Duration` from a JSON string containing a nanosecond count
pub(crate) fn parse_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    // TODO(tarcieri): handle 64-bit overflow?
    let nanos = String::deserialize(deserializer)?
        .parse::<u64>()
        .map_err(|e| D::Error::custom(format!("{}", e)))?;

    Ok(Duration::from_nanos(nanos))
}

/// Serialize `Duration` as a JSON string containing a nanosecond count
pub(crate) fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    format!("{}", duration.as_nanos()).serialize(serializer)
}

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

pub(crate) fn serialize_hex<S, T>(bytes: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    use serde::ser::Error;
    let hex_bytes = hex::encode(bytes.as_ref());
    let hex_string = String::from_utf8(hex_bytes).map_err(Error::custom)?;
    serializer.serialize_str(&hex_string)
}

// decode upper or lowercase hex
pub(crate) fn parse_hex<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let string = String::deserialize(deserializer)?;
    hex::decode_upper(&string)
        .or_else(|_| hex::decode(&string))
        .map_err(Error::custom)
}

pub(crate) fn serialize_base64<S, T>(bytes: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    use serde::ser::Error;
    let base64_bytes = base64::encode(bytes.as_ref());
    let base64_string = String::from_utf8(base64_bytes).map_err(Error::custom)?;
    serializer.serialize_str(&base64_string)
}

pub(crate) fn parse_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let string = String::deserialize(deserializer)?;
    base64::decode(&string).map_err(Error::custom)
}

pub(crate) fn serialize_option_base64<S>(
    maybe_bytes: &Option<Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    struct Wrapper<'a>(#[serde(serialize_with = "serialize_base64")] &'a Vec<u8>);

    match maybe_bytes {
        Some(bytes) => Wrapper(bytes).serialize(serializer),
        None => maybe_bytes.serialize(serializer),
    }
}

pub(crate) fn parse_option_base64<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "parse_base64")] Vec<u8>);

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
        #[serde(deserialize_with = "parse_u64")]
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
