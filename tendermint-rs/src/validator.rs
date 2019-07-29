//! Tendermint validators

use crate::{account, vote, PublicKey};
#[cfg(feature = "serde")]
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "rpc")]
use subtle_encoding::base64;

/// Validator information
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
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

/// Updates to the validator set
#[cfg(feature = "rpc")]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Update {
    /// Validator public key
    #[serde(deserialize_with = "deserialize_public_key")]
    pub pub_key: PublicKey,

    /// New voting power
    pub power: vote::Power,
}

/// Validator updates use a slightly different public key format than the one
/// implemented in `tendermint::PublicKey`.
///
/// This is an internal thunk type to parse the `validator_updates` format and
/// then convert to `tendermint::PublicKey` in `deserialize_public_key` below.
#[cfg(feature = "rpc")]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum PK {
    /// Ed25519 keys
    #[serde(rename = "ed25519")]
    Ed25519(String),
}

#[cfg(feature = "rpc")]
fn deserialize_public_key<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    match &PK::deserialize(deserializer)? {
        PK::Ed25519(base64_value) => {
            let bytes =
                base64::decode(base64_value).map_err(|e| D::Error::custom(format!("{}", e)))?;

            PublicKey::from_raw_ed25519(&bytes)
                .ok_or_else(|| D::Error::custom("error parsing Ed25519 key"))
        }
    }
}
