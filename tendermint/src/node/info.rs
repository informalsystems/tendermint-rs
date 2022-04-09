//! Node information (used in RPC responses)

use core::fmt;
use core::str::FromStr as _;

use serde::{Deserialize, Serialize};

use proto::Protobuf;
use tendermint_proto as proto;

use crate::prelude::*;
use crate::{chain, node, serializers, Moniker, Version};

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
    pub channels: Vec<u8>,

    /// Moniker
    pub moniker: Moniker,

    /// Other status information
    pub other: OtherInfo,
}

impl From<Info> for proto::p2p::NodeInfo {
    fn from(info: Info) -> Self {
        Self {
            channels: info.channels,
            listen_addr: info.listen_addr.to_string(),
            moniker: info.moniker.to_string(),
            network: info.network.to_string(),
            node_id: info.id.to_string(),
            other: Some(proto::p2p::NodeInfoOther::from(info.other)),
            protocol_version: Some(proto::p2p::ProtocolVersion::from(info.protocol_version)),
            version: info.version.to_string(),
        }
    }
}

impl TryFrom<proto::p2p::NodeInfo> for Info {
    type Error = crate::Error;

    fn try_from(input: proto::p2p::NodeInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            channels: input.channels,
            listen_addr: ListenAddress::new(input.listen_addr),
            id: node::Id::from_str(&input.node_id)?,
            network: chain::Id::try_from(input.network)?,
            version: Version::unchecked(input.version),
            moniker: Moniker::from_str(&input.moniker)?,
            other: OtherInfo::try_from(input.other.unwrap())?,
            protocol_version: ProtocolVersionInfo::try_from(input.protocol_version.unwrap())?,
        })
    }
}

impl Protobuf<proto::p2p::NodeInfo> for Info {}

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

impl From<ProtocolVersionInfo> for proto::p2p::ProtocolVersion {
    fn from(info: ProtocolVersionInfo) -> Self {
        Self {
            p2p: info.p2p,
            block: info.block,
            app: info.app,
        }
    }
}

impl TryFrom<proto::p2p::ProtocolVersion> for ProtocolVersionInfo {
    type Error = crate::Error;

    fn try_from(input: proto::p2p::ProtocolVersion) -> Result<Self, Self::Error> {
        Ok(Self {
            p2p: input.p2p,
            block: input.block,
            app: input.app,
        })
    }
}

impl Protobuf<proto::p2p::ProtocolVersion> for ProtocolVersionInfo {}

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

impl fmt::Display for ListenAddress {
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

impl From<OtherInfo> for proto::p2p::NodeInfoOther {
    fn from(info: OtherInfo) -> Self {
        Self {
            tx_index: info.tx_index.to_string(),
            rpc_address: info.rpc_address,
        }
    }
}

impl TryFrom<proto::p2p::NodeInfoOther> for OtherInfo {
    type Error = crate::Error;

    fn try_from(input: proto::p2p::NodeInfoOther) -> Result<Self, Self::Error> {
        let tx_index = match input.tx_index.as_str() {
            "on" => TxIndexStatus::On,
            "off" => TxIndexStatus::Off,
            _ => return Err(crate::Error::unsupported_tx_index_value()),
        };

        Ok(Self {
            tx_index,
            rpc_address: input.rpc_address,
        })
    }
}

impl Protobuf<proto::p2p::NodeInfoOther> for OtherInfo {}

/// Transaction index status
#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TxIndexStatus {
    /// Index is on
    #[serde(rename = "on")]
    On,

    /// Index is off
    #[serde(rename = "off")]
    Off,
}

impl Default for TxIndexStatus {
    fn default() -> TxIndexStatus {
        TxIndexStatus::On
    }
}

impl fmt::Display for TxIndexStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            Self::On => "on",
            Self::Off => "off",
        };
        write!(f, "{}", val)
    }
}

impl From<TxIndexStatus> for bool {
    fn from(status: TxIndexStatus) -> bool {
        match status {
            TxIndexStatus::On => true,
            TxIndexStatus::Off => false,
        }
    }
}
