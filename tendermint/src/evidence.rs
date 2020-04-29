//! Evidence of malfeasance by validators (i.e. signing conflicting votes).

use std::slice;
use {
    crate::{serializers, PublicKey, Vote},
    serde::{Deserialize, Serialize},
};

/// Evidence of malfeasance by validators (i.e. signing conflicting votes).
/// encoded using an Amino prefix. There is currently only a single type of
/// evidence: `DuplicateVoteEvidence`.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#evidence>
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Evidence {
    /// Duplicate vote evidence
    #[serde(rename = "tendermint/DuplicateVoteEvidence")]
    DuplicateVote(DuplicateVoteEvidence),
}

/// Duplicate vote evidence
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DuplicateVoteEvidence {
    #[serde(rename = "PubKey")]
    pub_key: PublicKey,
    #[serde(rename = "VoteA")]
    vote_a: Vote,
    #[serde(rename = "VoteB")]
    vote_b: Vote,
}

/// Evidence data is a wrapper for a list of `Evidence`.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#evidencedata>
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Data {
    evidence: Option<Vec<Evidence>>,
}

impl Data {
    /// Create a new evidence data collection
    pub fn new<I>(into_evidence: I) -> Data
    where
        I: Into<Vec<Evidence>>,
    {
        Data {
            evidence: Some(into_evidence.into()),
        }
    }

    /// Convert this evidence data into a vector
    pub fn into_vec(self) -> Vec<Evidence> {
        self.iter().cloned().collect()
    }

    /// Iterate over the evidence data
    pub fn iter(&self) -> slice::Iter<'_, Evidence> {
        self.as_ref().iter()
    }
}

impl AsRef<[Evidence]> for Data {
    fn as_ref(&self) -> &[Evidence] {
        self.evidence.as_deref().unwrap_or_else(|| &[])
    }
}

/// Evidence collection parameters
#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Params {
    /// Maximum allowed age for evidence to be collected
    #[serde(
        serialize_with = "serializers::serialize_u64",
        deserialize_with = "serializers::parse_u64"
    )]
    pub max_age_num_blocks: u64,

    /// Max age duration
    pub max_age_duration: Duration,
}

/// Duration is a wrapper around std::time::Duration
/// essentially, to keep the usages look cleaner
/// i.e. you can avoid using serde annotations everywhere
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Duration(
    #[serde(
        serialize_with = "serializers::serialize_duration",
        deserialize_with = "serializers::parse_duration"
    )]
    std::time::Duration,
);

impl From<Duration> for std::time::Duration {
    fn from(d: Duration) -> std::time::Duration {
        d.0
    }
}
