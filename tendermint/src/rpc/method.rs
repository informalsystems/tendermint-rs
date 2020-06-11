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

    /// Subscribe to events over the websocket
    Subscribe,

    /// Broadcast evidence
    BroadcastEvidence,
}

impl Method {
    /// Get a static string which represents this method name
    pub fn as_str(self) -> &'static str {
        match self {
            Method::AbciInfo => "abci_info",
            Method::AbciQuery => "abci_query",
            Method::Block => "block",
            Method::BlockResults => "block_results",
            Method::Blockchain => "blockchain",
            Method::BroadcastTxAsync => "broadcast_tx_async",
            Method::BroadcastTxSync => "broadcast_tx_sync",
            Method::BroadcastTxCommit => "broadcast_tx_commit",
            Method::Commit => "commit",
            Method::Genesis => "genesis",
            Method::Health => "health",
            Method::NetInfo => "net_info",
            Method::Status => "status",
            Method::Validators => "validators",
            Method::Subscribe => "subscribe",
            Method::BroadcastEvidence => "broadcast_evidence",
        }
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match s {
            "abci_info" => Method::AbciInfo,
            "abci_query" => Method::AbciQuery,
            "block" => Method::Block,
            "block_results" => Method::BlockResults,
            "blockchain" => Method::Blockchain,
            "broadcast_tx_async" => Method::BroadcastTxAsync,
            "broadcast_tx_sync" => Method::BroadcastTxSync,
            "broadcast_tx_commit" => Method::BroadcastTxCommit,
            "commit" => Method::Commit,
            "genesis" => Method::Genesis,
            "health" => Method::Health,
            "net_info" => Method::NetInfo,
            "status" => Method::Status,
            "validators" => Method::Validators,
            "subscribe" => Method::Subscribe,
            "broadcast_evidence" => Method::BroadcastEvidence,
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
