//! Light client implementation as per the [Core Verification specification][1].
//!
//! [1]: https://github.com/informalsystems/tendermint-rs/blob/main/docs/spec/lightclient/verification/verification.md

use core::time::Duration;

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::types::TrustThreshold;

/// Verification parameters
#[derive(Copy, Clone, Debug, PartialEq, Eq, Display, Serialize, Deserialize)]
#[display(fmt = "{self:?}")]
pub struct Options {
    /// Defines what fraction of the total voting power of a known
    /// and trusted validator set is sufficient for a commit to be
    /// accepted going forward.
    pub trust_threshold: TrustThreshold,

    /// How long a validator set is trusted for (must be shorter than the chain's
    /// unbonding period)
    pub trusting_period: Duration,

    /// Correction parameter dealing with only approximately synchronized clocks.
    /// The local clock should always be ahead of timestamps from the blockchain; this
    /// is the maximum amount that the local clock may drift behind a timestamp from the
    /// blockchain.
    pub clock_drift: Duration,
}
