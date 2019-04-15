use crate::error::Error;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};

/// Block height for a particular chain (i.e. number of blocks created since
/// the chain began)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Height(pub u64);

impl Height {
    /// Parse height from the integer type used in Amino messages
    pub fn parse(n: i64) -> Result<Self, Error> {
        if n >= 0 {
            Ok(Height(n as u64))
        } else {
            Err(Error::OutOfRange)
        }
    }

    /// Get inner integer value. Alternative to `.0` or `.into()`
    pub fn value(self) -> u64 {
        self.0
    }

    /// Increment the block height by 1
    pub fn increment(self) -> Self {
        Height(self.0.checked_add(1).unwrap())
    }
}

impl Debug for Height {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "block::Height({})", self.0)
    }
}

impl Display for Height {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for Height {
    fn from(n: i64) -> Height {
        Self::parse(n).unwrap()
    }
}

impl From<u64> for Height {
    fn from(n: u64) -> Height {
        Height(n)
    }
}

impl From<Height> for u64 {
    fn from(height: Height) -> u64 {
        height.0
    }
}

impl From<Height> for i64 {
    fn from(height: Height) -> i64 {
        height.0 as i64
    }
}

/// Parse `block::Height` from a type
pub trait ParseHeight {
    /// Parse `block::Height`, or return an `Error` if parsing failed
    fn parse_block_height(&self) -> Result<Height, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_by_one() {
        assert_eq!(Height::default().increment().value(), 1);
    }
}
