//! RawCommitSig type for deserialization
use crate::{account, Signature, Time};
use serde::{Deserialize, Deserializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::str::FromStr;
use serde::de::Error;

// Implements decision: https://github.com/tendermint/tendermint/blob/master/docs/architecture/adr-025-commit.md#decision

/// indicate which BlockID the signature is for
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum BlockIDFlag {
    /// vote is not included in the Commit.Precommits
    BlockIDFlagAbsent = 1,
    /// voted for the Commit.BlockID
    BlockIDFlagCommit = 2,
    /// voted for nil
    BlockIDFlagNil = 3,
}

/// RawCommitSig struct for interim deserialization of JSON object
#[derive(Deserialize)]
pub struct RawCommitSig {
    /// indicate which BlockID the signature is for
    pub block_id_flag: BlockIDFlag,
    /// Validator Address
    // <<< Compatibility code for https://github.com/informalsystems/tendermint-rs/issues/260
    #[serde(default, deserialize_with = "emptystring_or_accountid")]
    pub validator_address: Option<account::Id>,
    // === Real code after compatibility issue is resolved
    /*
    pub validator_address: account::Id,
    */
    // >>> end of real code
    /// Timestamp
    #[serde(default)]
    pub timestamp: Option<Time>,
    /// Signature
    #[serde(default, deserialize_with = "option_signature")]
    pub signature: Option<Signature>,
}

fn option_signature<'de, D>(deserializer: D) -> Result<Option<Signature>, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(|x: Option<_>| x.unwrap_or(None))
}

// <<< Compatibility code for https://github.com/informalsystems/tendermint-rs/issues/260
fn emptystring_or_accountid<'de, D>(deserializer: D) -> Result<Option<account::Id>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    if string.is_empty() {
        Ok(None)
    } else {
        account::Id::from_str(&string)
            .map(|r| Some(r))
            .map_err(|e| D::Error::custom(format!("{}", e)))
    }
}
// === Real code after compatibility issue is resolved
// >>> end of real code
