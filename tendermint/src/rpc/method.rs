//! JSONRPC request methods

use super::Error;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

/// JSONRPC request methods.
///
/// Serialized as the "method" field of JSONRPC/HTTP requests.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Method {
    /// Get ABCI info
    AbciInfo,

    /// Get ABCI query
    AbciQuery,

    /// Get block info
    Block,

    /// Get ABCI results for a particular block
    BlockResults,

    /// Get blockchain info
    Blockchain,

    /// Broadcast transaction asynchronously
    BroadcastTxAsync,

    /// Broadcast transaction synchronously
    BroadcastTxSync,

    /// Broadcast transaction commit
    BroadcastTxCommit,

    /// Get commit info for a block
    Commit,

    /// Get genesis file
    Genesis,

    /// Get health info
    Health,

    /// Get network info
    NetInfo,

    /// Get node status
    Status,

    /// Get validator info for a block
    Validators,
}

impl Method {
    /// Get a static string which represents this method name
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AbciInfo => "abci_info",
            Self::AbciQuery => "abci_query",
            Self::Block => "block",
            Self::BlockResults => "block_results",
            Self::Blockchain => "blockchain",
            Self::BroadcastTxAsync => "broadcast_tx_async",
            Self::BroadcastTxSync => "broadcast_tx_sync",
            Self::BroadcastTxCommit => "broadcast_tx_commit",
            Self::Commit => "commit",
            Self::Genesis => "genesis",
            Self::Health => "health",
            Self::NetInfo => "net_info",
            Self::Status => "status",
            Self::Validators => "validators",
        }
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match s {
            "abci_info" => Self::AbciInfo,
            "abci_query" => Self::AbciQuery,
            "block" => Self::Block,
            "block_results" => Self::BlockResults,
            "blockchain" => Self::Blockchain,
            "broadcast_tx_async" => Self::BroadcastTxAsync,
            "broadcast_tx_sync" => Self::BroadcastTxSync,
            "broadcast_tx_commit" => Self::BroadcastTxCommit,
            "commit" => Self::Commit,
            "genesis" => Self::Genesis,
            "health" => Self::Health,
            "net_info" => Self::NetInfo,
            "status" => Self::Status,
            "validators" => Self::Validators,
            other => return Err(Error::method_not_found(other)),
        })
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for Method {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Method {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{}", e)))
    }
}
