//! Node information (used in RPC responses)

use core::{
    fmt::{self, Display},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use tendermint_proto::p2p::{
    NodeInfo as RawInfo, NodeInfoOther as RawOtherInfo, ProtocolVersion as RawProtocolVersion,
};

use crate::{
    chain, channel::Channels, error::Error, node, prelude::*, serializers, Moniker, Version,
};

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

impl TryFrom<RawInfo> for Info {
    type Error = Error;
    fn try_from(raw: RawInfo) -> Result<Self, Self::Error> {
        // TODO(erwan): my understand is that a `Channels` is 20 bytes wide
        // let's get this to compile and ask for feedback on how to do validation properly
        let channels: Channels = raw.channels.try_into()?;

        let protocol_version = if let Some(version) = raw.protocol_version {
            version.try_into()?
        } else {
            return Err(Error::missing_data()); // TODO(erwan): could do better than this error?
        };

        let other_info = if let Some(raw_info) = raw.other {
            raw_info.try_into()?
        } else {
            return Err(Error::missing_data()); // TODO(erwan): could do better than this error?
        };

        Ok(Info {
            protocol_version: protocol_version,
            id: node::Id::from_str(raw.node_id.as_str())?,
            listen_addr: ListenAddress::new(raw.listen_addr),
            // TODO(erwan): improve trait bound on that method
            network: chain::Id::try_from(raw.network)?,
            version: Version(raw.version),
            channels: channels,
            // TODO(erwan): what validation does a moniker need
            moniker: Moniker::from_str(&raw.moniker)?,
            other: other_info,
        })
    }
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

impl TryFrom<RawProtocolVersion> for ProtocolVersionInfo {
    type Error = Error;
    fn try_from(raw: RawProtocolVersion) -> Result<Self, Self::Error> {
        Ok(ProtocolVersionInfo {
            p2p: raw.p2p,
            block: raw.block,
            app: raw.app,
        })
    }
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

impl TryFrom<RawOtherInfo> for OtherInfo {
    type Error = Error;
    fn try_from(raw: RawOtherInfo) -> Result<Self, Self::Error> {
        Ok(OtherInfo {
            tx_index: raw.tx_index.try_into()?,
            rpc_address: raw.rpc_address,
        })
    }
}

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

impl TryFrom<String> for TxIndexStatus {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "on" => Ok(TxIndexStatus::On),
            "off" => Ok(TxIndexStatus::Off),
            _ => Err(Error::missing_data()), // TODO(erwan): use more appropriate error
        }
    }
}

impl Default for TxIndexStatus {
    fn default() -> TxIndexStatus {
        TxIndexStatus::On
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
