use tendermint::evidence;
use tendermint_proto::v0_38 as raw;

use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// The Event serialization in the latest RPC dialect is the canonical
/// serialization for the ABCI domain type.
pub use tendermint::abci::Event;

#[derive(Default, Clone)]
pub struct Dialect;

impl crate::dialect::Dialect for Dialect {
    type Event = Event;
    type Evidence = Evidence;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(into = "raw::types::Evidence", try_from = "raw::types::Evidence")]
pub struct Evidence(evidence::Evidence);

impl From<Evidence> for raw::types::Evidence {
    fn from(evidence: Evidence) -> Self {
        evidence.0.into()
    }
}

impl TryFrom<raw::types::Evidence> for Evidence {
    type Error = <evidence::Evidence as TryFrom<raw::types::Evidence>>::Error;

    fn try_from(value: raw::types::Evidence) -> Result<Self, Self::Error> {
        Ok(Self(evidence::Evidence::try_from(value)?))
    }
}

impl From<evidence::Evidence> for Evidence {
    fn from(evidence: evidence::Evidence) -> Self {
        Self(evidence)
    }
}
