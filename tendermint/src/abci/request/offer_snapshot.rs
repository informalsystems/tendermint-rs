use crate::prelude::*;

use bytes::Bytes;

use super::super::types::Snapshot;

// bring into scope for doc links
#[allow(unused)]
use super::ApplySnapshotChunk;

#[doc = include_str!("../doc/request-offersnapshot.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OfferSnapshot {
    /// The snapshot offered for restoration.
    pub snapshot: Snapshot,
    /// The light client verified app hash for this height.
    // XXX(hdevalence): replace with apphash
    pub app_hash: Bytes,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::{TryFrom, TryInto};
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<OfferSnapshot> for pb::RequestOfferSnapshot {
    fn from(offer_snapshot: OfferSnapshot) -> Self {
        Self {
            snapshot: Some(offer_snapshot.snapshot.into()),
            app_hash: offer_snapshot.app_hash,
        }
    }
}

impl TryFrom<pb::RequestOfferSnapshot> for OfferSnapshot {
    type Error = crate::Error;

    fn try_from(offer_snapshot: pb::RequestOfferSnapshot) -> Result<Self, Self::Error> {
        Ok(Self {
            snapshot: offer_snapshot
                .snapshot
                .ok_or("missing snapshot")?
                .try_into()?,
            app_hash: offer_snapshot.app_hash,
        })
    }
}

impl Protobuf<pb::RequestOfferSnapshot> for OfferSnapshot {}
