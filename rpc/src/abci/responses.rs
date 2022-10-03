//! ABCI response types used by the `/block_results` RPC endpoint.

use core::fmt::{self, Display};

use serde::{Deserialize, Deserializer, Serialize};
use tendermint::{abci, consensus, validator};

use crate::prelude::*;

/// Begin block response.
///
/// This type corresponds to the `ResponseBeginBlock` proto from:
///
/// <https://github.com/tendermint/tendermint/blob/develop/abci/types/types.proto>
// TODO(tarcieri): generate this automatically from the proto
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BeginBlock {
    /// Tags
    #[serde(default)]
    pub tags: Vec<abci::EventAttribute>,
}

/// End block response.
///
/// This type corresponds to the `ResponseEndBlock` proto from:
///
/// <https://github.com/tendermint/tendermint/blob/develop/abci/types/types.proto>
// TODO(tarcieri): generate this automatically from the proto
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndBlock {
    /// Validator updates
    #[serde(deserialize_with = "deserialize_validator_updates")]
    pub validator_updates: Vec<validator::Update>,

    /// New consensus params
    pub consensus_param_updates: Option<consensus::Params>,

    /// Tags
    #[serde(default)]
    pub tags: Vec<abci::EventAttribute>,
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
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
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
