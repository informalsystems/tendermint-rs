//! Define traits and instances for dealing with trust thresholds.

use std::fmt::{self, Debug, Display};

use crate::serializers;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// TrustThreshold defines how much of the total voting power of a known
/// and trusted validator set is sufficient for a commit to be
/// accepted going forward.
pub trait TrustThreshold: Copy + Clone + Debug + Serialize + DeserializeOwned {
    /// Check whether the given signed voting power is sufficient according to
    /// this trust threshold against the given total voting power.
    fn is_enough_power(&self, signed_voting_power: u64, total_voting_power: u64) -> bool;
}

/// TrustThresholdFraction defines what fraction of the total voting power of a known
/// and trusted validator set is sufficient for a commit to be
/// accepted going forward.
/// The [`Default::default()`] returns true, iff at least a third of the trusted
/// voting power signed (in other words at least one honest validator signed).
/// Some clients might require more than +1/3 and can implement their own
/// [`TrustThreshold`] which can be passed into all relevant methods.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustThresholdFraction {
    /// Numerator of the trust threshold fraction
    #[serde(with = "serializers::from_str")]
    pub numerator: u64,
    /// Numerator of the trust threshold fraction
    #[serde(with = "serializers::from_str")]
    pub denominator: u64,
}

impl TrustThresholdFraction {
    /// Constant for a trust threshold of 2/3.
    pub const TWO_THIRDS: Self = Self {
        numerator: 2,
        denominator: 3,
    };

    /// Instantiate a TrustThresholdFraction if the given denominator and
    /// numerator are valid.
    ///
    /// The parameters are valid iff `1/3 <= numerator/denominator <= 1`.
    /// In any other case we return `None`.
    pub fn new(numerator: u64, denominator: u64) -> Option<Self> {
        if numerator <= denominator && denominator > 0 && 3 * numerator >= denominator {
            Some(Self {
                numerator,
                denominator,
            })
        } else {
            None
        }
    }
}

impl TrustThreshold for TrustThresholdFraction {
    fn is_enough_power(&self, signed_voting_power: u64, total_voting_power: u64) -> bool {
        signed_voting_power * self.denominator > total_voting_power * self.numerator
    }
}

impl Default for TrustThresholdFraction {
    fn default() -> Self {
        Self::new(1, 3)
            .expect("initializing TrustThresholdFraction with valid fraction mustn't panic")
    }
}

impl Display for TrustThresholdFraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}
