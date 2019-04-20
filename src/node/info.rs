//! Node information (used in RPC responses)

use crate::{chain, channel::Channels, net, node, rpc, Moniker, Version};
use serde::{Deserialize, Serialize};

/// Node information
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Info {
    /// Protocol version information
    pub protocol_version: ProtocolVersionInfo,

    /// Node ID
    pub id: node::Id,

    /// Listen address
    pub listen_addr: net::Address,

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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProtocolVersionInfo {
    /// P2P protocol version
    #[serde(
        serialize_with = "rpc::response::serialize_u64",
        deserialize_with = "rpc::response::parse_u64"
    )]
    pub p2p: u64,

    /// Block version
    #[serde(
        serialize_with = "rpc::response::serialize_u64",
        deserialize_with = "rpc::response::parse_u64"
    )]
    pub block: u64,

    /// App version
    #[serde(
        serialize_with = "rpc::response::serialize_u64",
        deserialize_with = "rpc::response::parse_u64"
    )]
    pub app: u64,
}

/// Other information
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OtherInfo {
    /// TX index status
    pub tx_index: TxIndexStatus,

    /// RPC address
    pub rpc_address: net::Address,
}

/// Transaction index status
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum TxIndexStatus {
    /// Index is on
    #[serde(rename = "on")]
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
