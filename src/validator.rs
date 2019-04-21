//! Tendermint validators

use crate::{account, vote, PublicKey};
#[cfg(feature = "serde")]
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};

/// Validator information
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Info {
    /// Validator account address
    pub address: account::Id,

    /// Validator public key
    pub pub_key: PublicKey,

    /// Validator voting power
    pub voting_power: vote::Power,

    /// Validator proposer priority
    pub proposer_priority: Option<ProposerPriority>,
}

/// Proposer priority
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct ProposerPriority(i64);

impl ProposerPriority {
    /// Get the current voting power
    pub fn value(self) -> i64 {
        self.0
    }
}

impl From<ProposerPriority> for i64 {
    fn from(priority: ProposerPriority) -> i64 {
        priority.value()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ProposerPriority {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(ProposerPriority(
            String::deserialize(deserializer)?
                .parse()
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        ))
    }
}

#[cfg(feature = "serde")]
impl Serialize for ProposerPriority {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.to_string().serialize(serializer)
    }
}
