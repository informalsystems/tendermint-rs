use crate::prelude::*;

// bring into scope for doc links
#[allow(unused)]
use super::super::types::Snapshot;

#[doc = include_str!("../doc/response-offersnapshot.md")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum OfferSnapshot {
    /// Unknown result, abort all snapshot restoration
    Unknown = 0,
    /// Snapshot accepted, apply chunks
    Accept = 1,
    /// Abort all snapshot restoration
    Abort = 2,
    /// Reject this specific snapshot, try others
    Reject = 3,
    /// Reject all snapshots of this format, try others
    RejectFormat = 4,
    /// Reject all snapshots from the sender(s), try others
    RejectSender = 5,
}

impl Default for OfferSnapshot {
    fn default() -> Self {
        Self::Unknown
    }
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::TryFrom;
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<OfferSnapshot> for pb::ResponseOfferSnapshot {
    fn from(offer_snapshot: OfferSnapshot) -> Self {
        Self {
            result: offer_snapshot as i32,
        }
    }
}

impl TryFrom<pb::ResponseOfferSnapshot> for OfferSnapshot {
    type Error = crate::Error;

    fn try_from(offer_snapshot: pb::ResponseOfferSnapshot) -> Result<Self, Self::Error> {
        Ok(match offer_snapshot.result {
            0 => OfferSnapshot::Unknown,
            1 => OfferSnapshot::Accept,
            2 => OfferSnapshot::Abort,
            3 => OfferSnapshot::Reject,
            4 => OfferSnapshot::RejectFormat,
            5 => OfferSnapshot::RejectSender,
            _ => Err("unknown offer snapshot result code")?,
        })
    }
}

impl Protobuf<pb::ResponseOfferSnapshot> for OfferSnapshot {}
