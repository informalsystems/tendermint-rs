use crate::error::{Error, Kind};
use std::convert::TryInto;
use std::{
    convert::TryFrom,
    fmt::{self, Debug, Display},
    str::FromStr,
};

/// ValidatorIndex for a particular Vote
#[derive(Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ValidatorIndex(u32);

impl TryFrom<i32> for ValidatorIndex {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(ValidatorIndex(
            value.try_into().map_err(|_| Kind::NegativeValidatorIndex)?,
        ))
    }
}

impl From<ValidatorIndex> for i32 {
    fn from(value: ValidatorIndex) -> Self {
        value.value() as i32 // does not overflow. The value is <= i32::MAX
    }
}

impl TryFrom<u32> for ValidatorIndex {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > i32::MAX as u32 {
            return Err(Kind::IntegerOverflow.into());
        }
        Ok(ValidatorIndex(value))
    }
}

impl From<ValidatorIndex> for u32 {
    fn from(value: ValidatorIndex) -> Self {
        value.value()
    }
}

impl TryFrom<usize> for ValidatorIndex {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(ValidatorIndex(
            value.try_into().map_err(|_| Kind::IntegerOverflow)?,
        ))
    }
}

impl From<ValidatorIndex> for usize {
    fn from(value: ValidatorIndex) -> Self {
        value
            .value()
            .try_into()
            .expect("Integer overflow: system usize maximum smaller than i32 maximum")
    }
}

impl ValidatorIndex {
    /// Get inner integer value. Alternative to `.0` or `.into()`
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Debug for ValidatorIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vote::ValidatorIndex({})", self.0)
    }
}

impl Display for ValidatorIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ValidatorIndex {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        ValidatorIndex::try_from(
            s.parse::<u32>()
                .map_err(|_| Kind::Parse.context("validator index decode"))?,
        )
    }
}
