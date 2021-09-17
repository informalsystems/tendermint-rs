//! Voting power

use core::convert::{TryFrom, TryInto};
use core::fmt;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use crate::error::Error;
use crate::prelude::*;

/// Voting power
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct Power(u64);

impl fmt::Display for Power {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl TryFrom<i64> for Power {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Power(value.try_into().map_err(Error::negative_power)?))
    }
}

impl From<Power> for i64 {
    fn from(value: Power) -> Self {
        value.value() as i64
    }
}

impl TryFrom<u64> for Power {
    type Error = Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let _val: i64 = value.try_into().map_err(Error::integer_overflow)?;

        Ok(Power(value))
    }
}

impl From<Power> for u64 {
    fn from(value: Power) -> Self {
        value.value()
    }
}

impl From<u32> for Power {
    fn from(value: u32) -> Self {
        Power(value as u64)
    }
}

impl From<u16> for Power {
    fn from(value: u16) -> Self {
        Power(value as u64)
    }
}

impl From<u8> for Power {
    fn from(value: u8) -> Self {
        Power(value as u64)
    }
}

impl Power {
    /// Get the current voting power
    pub fn value(self) -> u64 {
        self.0
    }

    /// Is the current voting power zero?
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl<'de> Deserialize<'de> for Power {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Power(
            String::deserialize(deserializer)?
                .parse::<i64>()
                .map_err(|e| D::Error::custom(format!("{}", e)))?
                .try_into()
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        ))
    }
}

impl Serialize for Power {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let proto_int: i64 = (*self).into();
        proto_int.to_string().serialize(serializer)
    }
}
