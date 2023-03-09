//! Node information (used in RPC responses)

use core::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::{chain, channel::Channels, node, prelude::*, serializers, Moniker, Version};

/// Node information
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Info {
    /// Protocol version information
    pub protocol_version: ProtocolVersionInfo,

    /// Node ID
    pub id: node::Id,

    /// Listen address
    pub listen_addr: ListenAddress,

    /// Tendermint network / chain ID,
    pub network: chain::Id,

    /// Tendermint version
    pub version: Version,

    /// Channels
    pub channels: Channels,

    /// Moniker
    pub moniker: Moniker,

    /// Other status information
    pub other: OtherInfo,
}

/// Protocol version information
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProtocolVersionInfo {
    /// P2P protocol version
    #[serde(with = "serializers::from_str")]
    pub p2p: u64,

    /// Block version
    #[serde(with = "serializers::from_str")]
    pub block: u64,

    /// App version
    #[serde(with = "serializers::from_str")]
    pub app: u64,
}

/// Listen address information
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ListenAddress(String);

impl ListenAddress {
    /// Construct `ListenAddress`
    pub fn new(s: String) -> ListenAddress {
        ListenAddress(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ListenAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Other information
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OtherInfo {
    /// TX index status
    pub tx_index: TxIndexStatus,

    /// RPC address
    pub rpc_address: String,
}

/// Transaction index status
#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum TxIndexStatus {
    /// Index is on
    #[serde(rename = "on")]
    #[default]
    On,

    /// Index is off
    #[serde(rename = "off")]
    Off,
}

impl From<TxIndexStatus> for bool {
    fn from(status: TxIndexStatus) -> bool {
        match status {
            TxIndexStatus::On => true,
            TxIndexStatus::Off => false,
        }
    }
}
