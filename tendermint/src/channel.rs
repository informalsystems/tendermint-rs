//! Channels (RPC types)

mod id;

pub use self::id::Id;
use crate::serializers;
pub use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Channels
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Channel {
    /// Channel ID
    #[serde(rename = "ID")]
    pub id: Id,

    /// Capacity of the send queue
    #[serde(
        rename = "SendQueueCapacity",
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub send_queue_capacity: u64,

    /// Size of the send queue
    #[serde(
        rename = "SendQueueSize",
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub send_queue_size: u64,

    /// Priority value
    #[serde(
        rename = "Priority",
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub priority: u64,

    /// Amount of data recently sent
    #[serde(
        rename = "RecentlySent",
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub recently_sent: u64,
}

/// Channel collections
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Channels(String);

impl Display for Channels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
