//! Time serialization with validation

use crate::Time;
use serde::{Deserializer, Serializer};
use std::convert::TryFrom;
use tendermint_proto::google::protobuf::Timestamp;

/// Deserialize string into Time
pub fn deserialize<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp = tendermint_proto::serializers::option_timestamp::deserialize(deserializer)?
        .ok_or_else(|| serde::de::Error::custom(&""))?;
    Time::try_from(timestamp).map_err(serde::de::Error::custom)
}

/// Serialize from Time into string
pub fn serialize<S>(value: &Time, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let timestamp: Timestamp = value.clone().into();
    tendermint_proto::serializers::option_timestamp::serialize(&Some(timestamp), serializer)
}
