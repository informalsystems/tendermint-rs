//! Gas: abstract representation for the cost of resources used by nodes when
//! processing transactions.
//!
//! For more information, see:
//!
//! <https://tendermint.com/docs/spec/abci/apps.html#gas>

use crate::error::Error;
use crate::prelude::*;
use core::{
    fmt::{self, Display},
    str::FromStr,
};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

/// Gas: representation of transaction processing resource costs
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Gas(u64);

impl Gas {
    /// Get the inner integer value
    pub fn value(self) -> u64 {
        self.0
    }
}

impl From<u64> for Gas {
    fn from(amount: u64) -> Gas {
        Gas(amount)
    }
}

impl From<Gas> for u64 {
    fn from(gas: Gas) -> u64 {
        gas.0
    }
}

impl Display for Gas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Gas {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let res = s
            .parse::<u64>()
            .map_err(|e| Error::parse_int(s.to_string(), e))?
            .into();

        Ok(res)
    }
}

impl<'de> Deserialize<'de> for Gas {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{}", e)))
    }
}

impl Serialize for Gas {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
