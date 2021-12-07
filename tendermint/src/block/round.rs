use crate::error::Error;
use crate::prelude::*;
use core::convert::TryInto;
use core::{
    convert::TryFrom,
    fmt::{self, Debug, Display},
    str::FromStr,
};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

/// Block round for a particular chain
#[derive(Copy, Clone, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Round(u32);

impl TryFrom<i32> for Round {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(Round(value.try_into().map_err(Error::negative_round)?))
    }
}

impl From<Round> for i32 {
    fn from(value: Round) -> Self {
        value.value() as i32 // does not overflow. The value is <= i32::MAX
    }
}

impl TryFrom<u32> for Round {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let _val: i32 = value.try_into().map_err(Error::integer_overflow)?;

        Ok(Round(value))
    }
}

impl From<Round> for u32 {
    fn from(value: Round) -> Self {
        value.value()
    }
}

impl From<u16> for Round {
    fn from(value: u16) -> Self {
        Round(value as u32)
    }
}

impl From<u8> for Round {
    fn from(value: u8) -> Self {
        Round(value as u32)
    }
}

impl Round {
    /// Get inner integer value. Alternative to `.0` or `.into()`
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Increment the block round by 1
    pub fn increment(self) -> Self {
        Round::try_from(self.0.checked_add(1).expect("round overflow")).unwrap()
    }
}

impl Debug for Round {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "block::Round({})", self.0)
    }
}

impl Display for Round {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Round {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Round::try_from(
            s.parse::<u32>()
                .map_err(|e| Error::parse_int(s.to_string(), e))?,
        )
    }
}

impl<'de> Deserialize<'de> for Round {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{}", e)))
    }
}

impl Serialize for Round {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        u32::from(*self).to_string().serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_by_one() {
        assert_eq!(Round::default().increment().value(), 1);
    }

    #[test]
    fn avoid_try_unwrap_dance() {
        assert_eq!(
            Round::try_from(2_u32).unwrap().value(),
            Round::from(2_u16).value()
        );
    }
}
