//! `/consensus_state` endpoint JSON-RPC wrapper

use crate::Method;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use tendermint::block::{Height, Round};
use tendermint::{account, hash, Hash};

/// Get the current consensus state.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct Request;

impl Request {
    pub fn new() -> Self {
        Self {}
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> Method {
        Method::ConsensusState
    }
}

impl crate::SimpleRequest for Request {}

/// The current consensus state (UNSTABLE).
///
/// Currently based on https://github.com/tendermint/tendermint/blob/e820e68acd69737cfb63bc9ccca5f5450a42b5cf/consensus/types/round_state.go#L97
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    pub round_state: RoundState,
}

impl crate::Response for Response {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoundState {
    #[serde(rename = "height/round/step")]
    pub height_round_step: HeightRoundStep,

    // TODO(thane): Convert to timestamp
    pub start_time: String,

    #[serde(with = "hash::allow_empty")]
    pub proposal_block_hash: Hash,

    #[serde(with = "hash::allow_empty")]
    pub locked_block_hash: Hash,

    #[serde(with = "hash::allow_empty")]
    pub valid_block_hash: Hash,

    pub height_vote_set: HeightVoteSet,

    pub proposer: ValidatorInfo,
}

#[derive(Clone, Debug)]
pub struct HeightRoundStep {
    /// Current block height
    pub height: Height,
    /// Current consensus round
    pub round: Round,
    /// Current consensus step
    pub step: i8,
}

impl Serialize for HeightRoundStep {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hrs = format!(
            "{}/{}/{}",
            self.height.value(),
            self.round.value(),
            self.step
        );
        serializer.serialize_str(&hrs)
    }
}

impl<'de> Deserialize<'de> for HeightRoundStep {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hrs = String::deserialize(deserializer)?
            .split('/')
            .map(String::from)
            .collect::<Vec<String>>();
        if hrs.len() != 3 {
            return Err(serde::de::Error::custom(format!(
                "expected 3 components to height/round/step field, but got {}",
                hrs.len()
            )));
        }
        let height = Height::from_str(&hrs[0]).map_err(serde::de::Error::custom)?;
        let round = Round::from_str(&hrs[1]).map_err(serde::de::Error::custom)?;
        let step = i8::from_str(&hrs[2]).map_err(serde::de::Error::custom)?;
        Ok(Self {
            height,
            round,
            step,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeightVoteSet(Vec<RoundVotes>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundVotes {
    // A Tendermint node currently serializes this particular field as an
    // integer and not a string (unlike that which is expected from the `Round`
    // type).
    pub round: u32,
    pub prevotes: Vec<String>,
    pub prevotes_bit_array: String,
    pub precommits: Vec<String>,
    pub precommits_bit_array: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: account::Id,
    pub index: i32,
}
