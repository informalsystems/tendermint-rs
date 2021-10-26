//! JSON-RPC request methods

use crate::prelude::*;
use crate::Error;
use core::{
    fmt::{self, Display},
    str::FromStr,
};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

/// JSON-RPC request methods.
///
/// Serialized as the "method" field of JSON-RPC/HTTP requests.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Method {
    /// Get ABCI info
    AbciInfo,

    /// Get ABCI query
    AbciQuery,

    /// Get block info
    Block,

    /// Get ABCI results for a particular block
    BlockResults,

    /// Search for blocks by their BeginBlock and EndBlock events
    BlockSearch,

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

    /// Get consensus parameters
    ConsensusParams,

    /// Get consensus state
    ConsensusState,

    /// Get genesis file
    Genesis,

    /// Get health info
    Health,

    /// Get network info
    NetInfo,

    /// Get node status
    Status,

    /// Find transaction by hash
    Tx,

    /// Search for transactions with their results
    TxSearch,

    /// Get validator info for a block
    Validators,

    /// Subscribe to events
    Subscribe,

    /// Unsubscribe from events
    Unsubscribe,

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
            Method::BlockSearch => "block_search",
            Method::Blockchain => "blockchain",
            Method::BroadcastEvidence => "broadcast_evidence",
            Method::BroadcastTxAsync => "broadcast_tx_async",
            Method::BroadcastTxSync => "broadcast_tx_sync",
            Method::BroadcastTxCommit => "broadcast_tx_commit",
            Method::Commit => "commit",
            Method::ConsensusParams => "consensus_params",
            Method::ConsensusState => "consensus_state",
            Method::Genesis => "genesis",
            Method::Health => "health",
            Method::NetInfo => "net_info",
            Method::Status => "status",
            Method::Subscribe => "subscribe",
            Method::Tx => "tx",
            Method::TxSearch => "tx_search",
            Method::Unsubscribe => "unsubscribe",
            Method::Validators => "validators",
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
            "block_search" => Method::BlockSearch,
            "blockchain" => Method::Blockchain,
            "broadcast_evidence" => Method::BroadcastEvidence,
            "broadcast_tx_async" => Method::BroadcastTxAsync,
            "broadcast_tx_sync" => Method::BroadcastTxSync,
            "broadcast_tx_commit" => Method::BroadcastTxCommit,
            "commit" => Method::Commit,
            "consensus_params" => Method::ConsensusParams,
            "consensus_state" => Method::ConsensusState,
            "genesis" => Method::Genesis,
            "health" => Method::Health,
            "net_info" => Method::NetInfo,
            "status" => Method::Status,
            "subscribe" => Method::Subscribe,
            "tx" => Method::Tx,
            "tx_search" => Method::TxSearch,
            "unsubscribe" => Method::Unsubscribe,
            "validators" => Method::Validators,
            other => return Err(Error::method_not_found(other.to_string())),
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
