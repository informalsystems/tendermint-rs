use crate::error::{Error, Kind};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    convert::TryFrom,
    fmt::{self, Debug, Display},
    str::FromStr,
};

/// Tendermint version
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Version(String);

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Protocol(pub u64);

impl Protocol {
    /// Get inner integer value. Alternative to `.0` or `.into()`
    pub fn value(self) -> u64 {
        self.0
    }

}

impl Debug for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "block::Height({})", self.0)
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol(1)
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<i64> for Protocol {
    type Error = Error;

    fn try_from(n: i64) -> Result<Protocol, Error> {
        if n >= 0 {
            Ok(Protocol(n as u64))
        } else {
            Err(Kind::OutOfRange.into())
        }
    }
}

impl From<u64> for Protocol {
    fn from(n: u64) -> Protocol {
        Protocol(n)
    }
}

impl From<Protocol> for u64 {
    fn from(height: Protocol) -> u64 {
        height.0
    }
}

impl From<Protocol> for i64 {
    fn from(height: Protocol) -> i64 {
        height.0 as i64
    }
}

impl FromStr for Protocol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(s.parse::<u64>().map_err(|_| Kind::Parse)?.into())
    }
}

impl<'de> Deserialize<'de> for Protocol {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{}", e)))?)
    }
}

impl Serialize for Protocol {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
