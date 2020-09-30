//! Custom, legacy serializers

use crate::{block, Hash};
use serde::{de::Error as _, Deserialize, Deserializer};
use std::str::FromStr;

// Todo: Refactor the "Option"-based serializers below.
//  Most of them are not needed if the structs are defined well (with enums).

/// Option<Hash> deserialization
pub fn parse_non_empty_hash<'de, D>(deserializer: D) -> Result<Option<Hash>, D::Error>
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
pub fn parse_non_empty_block_id<'de, D>(deserializer: D) -> Result<Option<block::Id>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Parts {
        total: u64,
        hash: String,
    }
    #[derive(Deserialize)]
    struct BlockId {
        hash: String,
        parts: Parts,
    }
    if let Some(tmp_id) = <Option<BlockId>>::deserialize(deserializer)? {
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
                        total: tmp_id.parts.total as u32, // Todo: manage overflow
                        hash: Hash::from_str(&tmp_id.parts.hash)
                            .map_err(|err| D::Error::custom(format!("{}", err)))?,
                    })
                },
            }))
        }
    } else {
        Ok(None)
    }
}

/// Parse null as default
pub fn null_as_default<'de, D, T: Default + Deserialize<'de>>(
    deserializer: D,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(<Option<T>>::deserialize(deserializer)?.unwrap_or_default())
}
