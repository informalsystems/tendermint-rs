//! Votes

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

/// Voting power
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct Power(u64);

impl Power {
    /// Create a new Power
    pub const fn new(p: u64) -> Self {
        Self(p)
    }

    /// Get the current voting power
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Is the current voting power zero?
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<Power> for u64 {
    fn from(power: Power) -> Self {
        power.0
    }
}

impl<'de> Deserialize<'de> for Power {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(
            String::deserialize(deserializer)?
                .parse()
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        ))
    }
}

impl Serialize for Power {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.to_string().serialize(serializer)
    }
}
