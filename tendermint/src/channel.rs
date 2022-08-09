//! Channels (RPC types)

mod id;

use core::{
    convert::TryFrom,
    fmt::{self, Display},
};

use serde::{Deserialize, Serialize};

pub use self::id::Id;
use crate::{error::Error, prelude::*, serializers};

/// Channels
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Channel {
    /// Channel ID
    #[serde(rename = "ID")]
    pub id: Id,

    /// Capacity of the send queue
    #[serde(rename = "SendQueueCapacity", with = "serializers::from_str")]
    pub send_queue_capacity: u64,

    /// Size of the send queue
    #[serde(rename = "SendQueueSize", with = "serializers::from_str")]
    pub send_queue_size: u64,

    /// Priority value
    #[serde(rename = "Priority", with = "serializers::from_str")]
    pub priority: u64,

    /// Amount of data recently sent
    #[serde(rename = "RecentlySent", with = "serializers::from_str")]
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

impl TryFrom<Vec<u8>> for Channels {
    type Error = Error;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // TODO(erwan): what validation does a `Channels` need?
        // possibly: length should be 20 bytes?
        let value = String::from_utf8(value)
            .map_err(|_| Error::parse("failed parsing channels".to_string()))?;
        Ok(Channels(value))
    }
}
