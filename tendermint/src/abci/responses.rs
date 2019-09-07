//! ABCI response types used by the `/block_results` RPC endpoint.

use super::{code::Code, data::Data, gas::Gas, info::Info, log::Log, tag::Tag};
use crate::{consensus, validator};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{self, Display};

/// Responses for ABCI calls which occur during block processing.
///
/// Returned from the `/block_results` RPC endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Responses {
    /// Deliver TX response.
    // TODO(tarcieri): remove the `alias` attribute when this lands upstream:
    // <https://github.com/tendermint/tendermint/pull/3708/files>
    #[serde(alias = "DeliverTx")]
    #[serde(default, deserialize_with = "deserialize_deliver_tx")]
    pub deliver_tx: Vec<DeliverTx>,

    /// Begin block response.
    // TODO(tarcieri): remove the `alias` attribute when this lands upstream:
    // <https://github.com/tendermint/tendermint/pull/3708/files>
    #[serde(alias = "BeginBlock")]
    pub begin_block: Option<BeginBlock>,

    /// End block response.
    // TODO(tarcieri): remove the `alias` attribute when this lands upstream:
    // <https://github.com/tendermint/tendermint/pull/3708/files>
    #[serde(alias = "EndBlock")]
    pub end_block: Option<EndBlock>,
}

/// Return an empty vec in the event `deliver_tx` is `null`
fn deserialize_deliver_tx<'de, D>(deserializer: D) -> Result<Vec<DeliverTx>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}

/// Deliver TX response.
///
/// This type corresponds to the `ResponseDeliverTx` proto from:
///
/// <https://github.com/tendermint/tendermint/blob/develop/abci/types/types.proto>
// TODO(tarcieri): generate this automatically from the proto
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeliverTx {
    /// ABCI application response code
    pub code: Option<Code>,

    /// ABCI application data
    pub data: Option<Data>,

    /// ABCI log data (nondeterministic)
    pub log: Option<Log>,

    /// ABCI info (nondeterministic)
    pub info: Option<Info>,

    /// Amount of gas wanted
    #[serde(default, rename = "gasWanted")]
    pub gas_wanted: Gas,

    /// Amount of gas used
    #[serde(default, rename = "gasUsed")]
    pub gas_used: Gas,

    /// Tags
    #[serde(default)]
    pub tags: Vec<Tag>,

    /// Codespace
    pub codespace: Option<Codespace>,
}

/// Begin block response.
///
/// This type corresponds to the `ResponseBeginBlock` proto from:
///
/// <https://github.com/tendermint/tendermint/blob/develop/abci/types/types.proto>
// TODO(tarcieri): generate this automatically from the proto
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeginBlock {
    /// Tags
    #[serde(default)]
    pub tags: Vec<Tag>,
}

/// End block response.
///
/// This type corresponds to the `ResponseEndBlock` proto from:
///
/// <https://github.com/tendermint/tendermint/blob/develop/abci/types/types.proto>
// TODO(tarcieri): generate this automatically from the proto
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Return an empty vec in the event `validator_updates` is `null`
fn deserialize_validator_updates<'de, D>(
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
