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
    #[serde(rename = "SendQueueCapacity", with = "serializers::primitives::string")]
    pub send_queue_capacity: u64,

    /// Size of the send queue
    #[serde(rename = "SendQueueSize", with = "serializers::primitives::string")]
    pub send_queue_size: u64,

    /// Priority value
    #[serde(rename = "Priority", with = "serializers::primitives::string")]
    pub priority: u64,

    /// Amount of data recently sent
    #[serde(rename = "RecentlySent", with = "serializers::primitives::string")]
    pub recently_sent: u64,
}

/// Channel collections
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub struct Channels(String);

impl Display for Channels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
