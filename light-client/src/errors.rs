//! Toplevel errors raised by the light client.

use std::fmt::Debug;
use std::time::Duration;

use crate::operations::voting_power::VotingPowerTally;
use crossbeam_channel as crossbeam;

use crate::{
    components::io::IoError,
    light_client::Options,
    predicates::errors::VerificationErrorDetail,
    types::{Hash, Height, LightBlock, PeerId, Status},
};
use flex_error::{define_error, DisplayError, TraceError};

#[cfg(feature = "sled")]
type SledError = TraceError<sled::Error>;

#[cfg(not(feature = "sled"))]
type SledError = flex_error::NoSource;

define_error! {
    #[derive(Debug)]
    Error {
        Io
            [ IoError ]
            | _ | { "io error" },

        NoPrimary
            | _ | { "no primary" },

        NoWitnesses
            | _ | { "no witnesses" },

        NoWitnessesLeft
            | _ | { "no witnesses left" },

        ForkDetected
            { peers: Vec<PeerId> }
            | e | {
                format_args!("fork detected peers={0:?}",
                    e.peers)
            },

        NoInitialTrustedState
            | _ | { "no initial trusted state" },

        NoTrustedState
            { status: Status }
            | e | {
                format_args!("no trusted state with status {:?}",
                    e.status)
            },

        TargetLowerThanTrustedState
            {
                target_height: Height,
                trusted_height: Height,
            }
            | e | {
                format_args!("target height ({0}) is lower than trusted state ({1})",
                    e.target_height, e.trusted_height)
            },

        TrustedStateOutsideTrustingPeriod
            {
                trusted_state: Box<LightBlock>,
                options: Options,
            }
            | _ | {
                format_args!("trusted state outside of trusting period")
            },

        BisectionFailed
            {
                target_height: Height,
                trusted_height: Height
            }
            | e | {
                format_args!("bisection for target at height {0} failed when reached trusted state at height {1}",
                    e.target_height, e.trusted_height)
            },

        InvalidLightBlock
            [ DisplayError<VerificationErrorDetail> ]
            | _ | { "invalid light block" },

        InvalidAdjacentHeaders
            {
                hash1: Hash,
                hash2: Hash,
            }
            | e | {
                format_args!("hash mismatch between two adjacent headers: {0} != {1}",
                    e.hash1, e.hash2)
            },

        MissingLastBlockId
            { height: Height }
            | e | {
                format_args!("missing last_block_id for header at height {0}",
                    e.height)
            },

        ChannelDisconnected
            | _ | { "internal channel disconnected" },

        Sled
            [ SledError ]
            | _ | { "sled error" },

        SerdeCbor
            [ TraceError<serde_cbor::Error> ]
            | _ | { "serde cbor error" },

    }
}

/// Extension methods for `ErrorKind`
pub trait ErrorExt {
    /// Whether this error means that the light block
    /// cannot be trusted w.r.t. the latest trusted state.
    fn not_enough_trust(&self) -> Option<VotingPowerTally>;

    /// Whether this error means that the light block has expired,
    /// ie. it's outside of the trusting period.
    fn has_expired(&self) -> bool;

    /// Whether this error means that a timeout occured when
    /// querying a node.
    fn is_timeout(&self) -> Option<Duration>;
}

impl ErrorExt for ErrorDetail {
    fn not_enough_trust(&self) -> Option<VotingPowerTally> {
        if let Self::InvalidLightBlock(e) = self {
            e.source.not_enough_trust()
        } else {
            None
        }
    }

    fn has_expired(&self) -> bool {
        if let Self::InvalidLightBlock(e) = self {
            e.source.has_expired()
        } else {
            false
        }
    }

    /// Whether this error means that a timeout occured when querying a node.
    fn is_timeout(&self) -> Option<Duration> {
        if let Self::Io(e) = self {
            e.source.is_timeout()
        } else {
            None
        }
    }
}

pub fn send_error<T>(_e: crossbeam::SendError<T>) -> Error {
    channel_disconnected_error()
}

pub fn recv_error(_e: crossbeam::RecvError) -> Error {
    channel_disconnected_error()
}
