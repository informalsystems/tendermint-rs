//! `/status` endpoint JSONRPC wrapper

use crate::{account, block, node, rpc, Hash, PublicKey, Timestamp};
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};

/// Node status request
#[derive(Debug, Default)]
pub struct Request;

impl rpc::Request for Request {
    type Response = Response;

    fn path(&self) -> rpc::request::Path {
        "/status".parse().unwrap()
    }
}

/// Status responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Node information
    pub node_info: node::Info,

    /// Sync information
    pub sync_info: SyncInfo,

    /// Validator information
    pub validator_info: ValidatorInfo,
}

impl rpc::Response for Response {}

/// Sync information
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncInfo {
    /// Latest block hash
    pub latest_block_hash: Hash,

    /// Latest app hash
    pub latest_app_hash: Hash,

    /// Latest block height
    pub latest_block_height: block::Height,

    /// Latest block time
    pub latest_block_time: Timestamp,

    /// Are we catching up?
    pub catching_up: bool,
}

/// Validator information
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidatorInfo {
    /// Validator account address
    pub address: account::Id,

    /// Validator public key
    pub pub_key: PublicKey,

    /// Validator voting power
    pub voting_power: VotingPower,
}

/// Voting power
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct VotingPower(u64);

impl VotingPower {
    /// Get the current voting power
    pub fn value(self) -> u64 {
        self.0
    }

    /// Is the current voting power zero?
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<VotingPower> for u64 {
    fn from(power: VotingPower) -> u64 {
        power.0
    }
}

impl<'de> Deserialize<'de> for VotingPower {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(VotingPower(
            String::deserialize(deserializer)?
                .parse()
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        ))
    }
}

impl Serialize for VotingPower {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.to_string().serialize(serializer)
    }
}
