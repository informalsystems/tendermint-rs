//! Block size parameters

use {
    crate::serializers,
    serde::{Deserialize, Serialize},
};

/// Block size parameters
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Size {
    /// Maximum number of bytes in a block
    #[serde(
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub max_bytes: u64,

    /// Maximum amount of gas which can be spent on a block
    #[serde(
        serialize_with = "serializers::serialize_i64",
        deserialize_with = "serializers::parse_i64"
    )]
    pub max_gas: i64,

    /// Time iota in ms
    #[serde(
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub time_iota_ms: u64,
}
