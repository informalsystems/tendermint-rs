//! Define traits and instances for dealing with trust thresholds.

use core::fmt::{self, Debug, Display};

use crate::error::Error;
use crate::prelude::*;
use crate::serializers;
use core::convert::TryFrom;
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
#[serde(
    try_from = "RawTrustThresholdFraction",
    into = "RawTrustThresholdFraction"
)]
pub struct TrustThresholdFraction {
    numerator: u64,
    denominator: u64,
}

impl TrustThresholdFraction {
    /// Constant for a trust threshold of 1/3.
    pub const ONE_THIRD: Self = Self {
        numerator: 1,
        denominator: 3,
    };

    /// Constant for a trust threshold of 2/3.
    pub const TWO_THIRDS: Self = Self {
        numerator: 2,
        denominator: 3,
    };

    /// Instantiate a TrustThresholdFraction if the given denominator and
    /// numerator are valid.
    ///
    /// The parameters are valid iff `1/3 <= numerator/denominator < 1`.
    /// In any other case we return an error.
    pub fn new(numerator: u64, denominator: u64) -> Result<Self, Error> {
        if numerator >= denominator {
            return Err(Error::trust_threshold_too_large());
        }
        if denominator == 0 {
            return Err(Error::undefined_trust_threshold());
        }
        if 3 * numerator < denominator {
            return Err(Error::trust_threshold_too_small());
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    /// The numerator of this fraction.
    pub fn numerator(&self) -> u64 {
        self.numerator
    }

    /// The denominator of this fraction.
    pub fn denominator(&self) -> u64 {
        self.denominator
    }
}

impl TryFrom<RawTrustThresholdFraction> for TrustThresholdFraction {
    type Error = Error;

    fn try_from(value: RawTrustThresholdFraction) -> Result<Self, Self::Error> {
        Self::new(value.numerator, value.denominator)
    }
}

impl From<TrustThresholdFraction> for RawTrustThresholdFraction {
    fn from(f: TrustThresholdFraction) -> Self {
        Self {
            numerator: f.numerator,
            denominator: f.denominator,
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
        Self::ONE_THIRD
    }
}

impl Display for TrustThresholdFraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

/// Facilitates validation of [`TrustThresholdFraction`] instances when
/// deserializing them.
#[derive(Serialize, Deserialize)]
pub struct RawTrustThresholdFraction {
    #[serde(with = "serializers::from_str")]
    numerator: u64,
    #[serde(with = "serializers::from_str")]
    denominator: u64,
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    fn to_json(num: u64, denom: u64) -> String {
        format!(
            "{{\"numerator\": \"{}\", \"denominator\": \"{}\"}}",
            num, denom
        )
    }

    fn from_json(num: u64, denom: u64) -> Result<TrustThresholdFraction, serde_json::Error> {
        let json = to_json(num, denom);
        serde_json::from_str::<TrustThresholdFraction>(&json)
    }

    prop_compose! {
        // num < denom <= 3*num
        fn arb_correct_frac(num: u64)(denom in (num+1)..=(3*num)) -> (u64, u64) {
            (num, denom)
        }
    }

    proptest! {
        #[test]
        fn too_large(num in 2..1000u64) {
            // Just smaller than the numerator
            let denom = num - 1;
            assert!(TrustThresholdFraction::new(num, denom).is_err());
            assert!(from_json(num, denom).is_err());
        }

        #[test]
        fn cannot_be_one(num in 1..1000u64) {
            assert!(TrustThresholdFraction::new(num, num).is_err());
            assert!(from_json(num, num).is_err());
        }

        #[test]
        fn undefined(num in 1..1000u64) {
            // Numerator should be irrelevant
            let denom = 0u64;
            assert!(TrustThresholdFraction::new(num, denom).is_err());
            assert!(from_json(num, denom).is_err());
        }

        #[test]
        fn too_small(num in 1..1000u64) {
            // Just larger than 3 times the numerator
            let denom = (num * 3) + 1;
            assert!(TrustThresholdFraction::new(num, denom).is_err());
            assert!(from_json(num, denom).is_err());
        }

        #[test]
        fn just_right((num, denom) in arb_correct_frac(1000)) {
            let frac = TrustThresholdFraction::new(num, denom).unwrap();
            assert_eq!(frac.numerator(), num);
            assert_eq!(frac.denominator(), denom);

            let frac = from_json(num, denom).unwrap();
            assert_eq!(frac.numerator(), num);
            assert_eq!(frac.denominator(), denom);
        }
    }
}
