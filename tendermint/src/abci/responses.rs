//! ABCI response types used by the `/block_results` RPC endpoint.

use super::{
    code::Code, data::Data, gas::Gas, info::Info, log::Log, tag::Tag, transaction::Transaction,
};
use crate::{consensus, serializers, validator};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{self, Display};

/// Responses for ABCI calls which occur during block processing.
///
/// Returned from the `/block_results` RPC endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Responses {
    /// Finalize block response.
    // TODO(tarcieri): remove the `alias` attribute when this lands upstream:
    // <https://github.com/tendermint/tendermint/pull/3708/files>
    #[serde(alias = "FinalizeBlock")]
    pub finalize_block: Option<FinalizeBlock>,
}

/// Deliver TX response.
///
/// This type corresponds to the `ResponseDeliverTx` proto from:
///
/// <https://github.com/tendermint/tendermint/blob/master/abci/types/types.proto>
// TODO(tarcieri): generate this automatically from the proto
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeliverTx {
    /// ABCI application response code
    pub code: Code,

    /// ABCI application data
    #[serde(with = "serializers::nullable")]
    pub data: Data,

    /// ABCI log data (nondeterministic)
    pub log: Log,

    /// ABCI info (nondeterministic)
    pub info: Info,

    /// Amount of gas wanted
    #[serde(default, rename = "gasWanted")]
    pub gas_wanted: Gas,

    /// Amount of gas used
    #[serde(default, rename = "gasUsed")]
    pub gas_used: Gas,

    /// Events
    pub events: Vec<Event>,

    /// Codespace
    pub codespace: Codespace,
}

/// Event
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    /// Event type
    #[serde(rename = "type")]
    pub type_str: String,

    /// Attributes
    pub attributes: Vec<Tag>,
}

// type alias for however data is internally stored within the consensus engine.
type BlockData = Vec<Transaction>;

/// Prepare proposal response.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PrepareProposal {
    /// Tags
    #[serde(default)]
    pub blockdata: BlockData,
    //FIXME(Ash): add header field
}

type Evidence = crate::evidence::Evidence;

/// Verify header response.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct VerifyHeader {
    accept_header: bool,
    evidence: Vec<Evidence>,
}

/// Process proposal response.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct ProcessProposal {
    accept_block: bool,
    evidence: Vec<Evidence>,
}

/// End block response.
///
/// This type corresponds to the `ResponseEndBlock` proto from:
///
/// <https://github.com/tendermint/tendermint/blob/develop/abci/types/types.proto>
// TODO(tarcieri): generate this automatically from the proto
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EndBlock {
    /// Validator updates
    #[serde(deserialize_with = "deserialize_validator_updates")]
    pub validator_updates: Vec<validator::Update>,

    /// New consensus params
    pub consensus_param_updates: Option<consensus::Params>,

    /// Tags
    #[serde(default)]
    pub tags: Vec<Tag>,
}

/// Finalize block response.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FinalizeBlock {
    /// updates resulting from block finalization
    pub updates: EndBlock,
    /// resulting txs from block finalization
    pub tx_results: Vec<DeliverTx>,
}

/// Extend vote response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtendVote {
    /// updates resulting from block finalization
    pub unsigned_app_vote_data: Vec<u8>,
    /// resulting txs from block finalization
    pub self_authenticating_app_data: Vec<u8>,
}

/// Return an empty vec in the event `validator_updates` is `null`
pub fn deserialize_validator_updates<'de, D>(
    deserializer: D,
) -> Result<Vec<validator::Update>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}

/// Codespace
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Codespace(String);

impl AsRef<str> for Codespace {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for Codespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for Codespace {
    fn default() -> Self {
        Self(String::new())
    }
}
