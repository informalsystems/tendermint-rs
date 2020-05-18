//! Custom, legacy serializers

use crate::account::{Id, LENGTH};
use crate::{block, Hash, Signature};
use serde::{de::Error as _, Deserialize, Deserializer};
use std::str::FromStr;

// Todo: Refactor the "Option"-based serializers below.
//  Most of them are not needed if the structs are defined well (with enums).

/// Option<Hash> deserialization
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

/// Parse empty block id as None.
pub(crate) fn parse_non_empty_block_id<'de, D>(
    deserializer: D,
) -> Result<Option<block::Id>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Parts {
        #[serde(with = "super::from_str")]
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

/// Option<Id> deserialization
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

/// Option<Signature> deserialization
pub(crate) fn parse_non_empty_signature<'de, D>(
    deserializer: D,
) -> Result<Option<Signature>, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(|x: Option<_>| x.unwrap_or(None))
}
