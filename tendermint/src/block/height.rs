use crate::error::{Error, ErrorKind};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    convert::TryFrom,
    fmt::{self, Debug, Display},
    str::FromStr,
};

/// Block height for a particular chain (i.e. number of blocks created since
/// the chain began)
/// 
/// A height of 0 represents a chain which has not yet produced a block.
#[derive(Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Height(pub u64);

impl Height {
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

impl Default for Height {
    fn default() -> Self {
        Height(1)
    }
}

impl Display for Height {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<i64> for Height {
    type Error = Error;

    fn try_from(n: i64) -> Result<Height, Error> {
        if n >= 0 {
            Ok(Height(n as u64))
        } else {
            Err(ErrorKind::OutOfRange.into())
        }
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

impl FromStr for Height {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(s.parse::<u64>().map_err(|_| ErrorKind::Parse)?.into())
    }
}

impl<'de> Deserialize<'de> for Height {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{}", e)))?)
    }
}

impl Serialize for Height {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
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
        assert_eq!(Height::default().increment().value(), 2);
    }
}
