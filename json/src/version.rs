//! App/consensus version-related data structures for Tendermint.

use crate::serializers;
use serde::{Deserialize, Serialize};

/// App includes the protocol and software version for the application.
/// This information is included in ResponseInfo. The App.Protocol can be
/// updated in ResponseEndBlock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct App {
    pub protocol: u64,
    pub software: String,
}

/// Consensus captures the consensus rules for processing a block in the blockchain,
/// including all blockchain data structures and the rules of the application's
/// state transition machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Consensus {
    #[serde(with = "serializers::from_str")]
    pub block: u64,
    #[serde(with = "serializers::from_str", default)]
    pub app: u64,
}
