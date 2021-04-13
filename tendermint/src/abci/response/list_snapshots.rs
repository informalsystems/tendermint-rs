use crate::prelude::*;

use super::super::types::Snapshot;

#[doc = include_str!("../doc/response-listsnapshots.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ListSnapshots {
    /// A list of local state snapshots.
    pub snapshots: Vec<Snapshot>,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::{TryFrom, TryInto};
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<ListSnapshots> for pb::ResponseListSnapshots {
    fn from(list_snapshots: ListSnapshots) -> Self {
        Self {
            snapshots: list_snapshots
                .snapshots
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl TryFrom<pb::ResponseListSnapshots> for ListSnapshots {
    type Error = crate::Error;

    fn try_from(list_snapshots: pb::ResponseListSnapshots) -> Result<Self, Self::Error> {
        Ok(Self {
            snapshots: list_snapshots
                .snapshots
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Protobuf<pb::ResponseListSnapshots> for ListSnapshots {}
