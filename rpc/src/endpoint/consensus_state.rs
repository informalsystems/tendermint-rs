//! `/consensus_state` endpoint JSON-RPC wrapper

use crate::{Error, Method};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use subtle_encoding::hex;
use tendermint::block::{Height, Round};
use tendermint::{account, hash, vote, Hash, Time};

// From https://github.com/tendermint/tendermint/blob/e820e68acd69737cfb63bc9ccca5f5450a42b5cf/types/vote.go#L16
const NIL_VOTE_STR: &str = "nil-Vote";

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

/// The state of a particular consensus round.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoundState {
    #[serde(alias = "height/round/step")]
    pub height_round_step: HeightRoundStep,

    #[serde(with = "tendermint::serializers::time")]
    pub start_time: Time,

    #[serde(with = "hash::allow_empty")]
    pub proposal_block_hash: Hash,

    #[serde(with = "hash::allow_empty")]
    pub locked_block_hash: Hash,

    #[serde(with = "hash::allow_empty")]
    pub valid_block_hash: Hash,

    pub height_vote_set: Vec<RoundVotes>,

    pub proposer: ValidatorInfo,
}

/// A compound object indicating a height, round and step for consensus state.
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
        let s = String::deserialize(deserializer)?;
        let hrs: Vec<&str> = s.split('/').collect();
        if hrs.len() != 3 {
            return Err(serde::de::Error::custom(format!(
                "expected 3 components to height/round/step field, but got {}",
                hrs.len()
            )));
        }
        let height = Height::from_str(hrs[0]).map_err(serde::de::Error::custom)?;
        let round = Round::from_str(hrs[1]).map_err(serde::de::Error::custom)?;
        let step = i8::from_str(hrs[2]).map_err(serde::de::Error::custom)?;
        Ok(Self {
            height,
            round,
            step,
        })
    }
}

/// Details of all votes for a particular consensus round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundVotes {
    // A Tendermint node currently serializes this particular field as an
    // integer and not a string (unlike that which is expected from the `Round`
    // type).
    pub round: u32,
    pub prevotes: Vec<RoundVote>,
    pub prevotes_bit_array: String,
    pub precommits: Vec<RoundVote>,
    pub precommits_bit_array: String,
}

/// Details of a single vote from a particular consensus round.
#[derive(Debug, Clone, PartialEq)]
pub enum RoundVote {
    Nil,
    Vote(VoteSummary),
}

impl Serialize for RoundVote {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RoundVote::Nil => serializer.serialize_str(NIL_VOTE_STR),
            RoundVote::Vote(summary) => serializer.serialize_str(&summary.to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for RoundVote {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == NIL_VOTE_STR {
            Ok(Self::Nil)
        } else {
            Ok(Self::Vote(
                VoteSummary::from_str(&s).map_err(serde::de::Error::custom)?,
            ))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoteSummary {
    pub validator_index: i32,
    pub validator_address_fingerprint: Fingerprint,
    pub height: Height,
    pub round: Round,
    pub vote_type: vote::Type,
    pub block_id_hash_fingerprint: Fingerprint,
    pub signature_fingerprint: Fingerprint,
    pub timestamp: Time,
}

impl FromStr for VoteSummary {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s
            .strip_prefix("Vote{")
            .ok_or_else(|| {
                Self::Err::client_internal_error(
                    "invalid format for consensus state vote summary string",
                )
            })?
            .strip_suffix('}')
            .ok_or_else(|| {
                Self::Err::client_internal_error(
                    "invalid format for consensus state vote summary string",
                )
            })?
            .split(' ')
            .collect();
        if parts.len() != 6 {
            return Err(Self::Err::client_internal_error(format!(
                "expected 6 parts to a consensus state vote summary, but got {}",
                parts.len()
            )));
        }
        let validator: Vec<&str> = parts[0].split(':').collect();
        if validator.len() != 2 {
            return Err(Self::Err::client_internal_error(format!(
                "failed to parse validator info for consensus state vote summary: {}",
                parts[0],
            )));
        }
        let height_round_type: Vec<&str> = parts[1].split('/').collect();
        if height_round_type.len() != 3 {
            return Err(Self::Err::client_internal_error(format!(
                "failed to parse height/round/type for consensus state vote summary: {}",
                parts[1]
            )));
        }

        let validator_index = i32::from_str(validator[0]).map_err(|e| {
            Self::Err::client_internal_error(format!(
                "failed to parse validator index from consensus state vote summary: {} ({})",
                e, validator[0],
            ))
        })?;
        let validator_address_fingerprint =
            Fingerprint::from_str(validator[1]).map_err(|e| {
                Self::Err::client_internal_error(format!(
                    "failed to parse validator address fingerprint from consensus state vote summary: {}",
                    e
                ))
            })?;
        let height = Height::from_str(height_round_type[0]).map_err(|e| {
            Self::Err::client_internal_error(format!(
                "failed to parse height from consensus state vote summary: {}",
                e
            ))
        })?;
        let round = Round::from_str(height_round_type[1]).map_err(|e| {
            Self::Err::client_internal_error(format!(
                "failed to parse round from consensus state vote summary: {}",
                e
            ))
        })?;
        let vote_type_num: Vec<&str> = height_round_type[2].split('(').collect();
        let vote_type_id = i32::from_str(vote_type_num[0]).map_err(|e| {
            Self::Err::client_internal_error(format!(
                "failed to parse vote type from consensus state vote summary: {} ({})",
                e, vote_type_num[0],
            ))
        })?;
        let vote_type = vote::Type::try_from(vote_type_id).map_err(|e| {
            Self::Err::client_internal_error(format!(
                "failed to parse vote type from consensus state vote summary: {} ({})",
                e, vote_type_id,
            ))
        })?;
        let block_id_hash_fingerprint = Fingerprint::from_str(parts[2]).map_err(|e| {
            Self::Err::client_internal_error(format!(
                "failed to parse block ID hash fingerprint from consensus state vote summary: {}",
                e
            ))
        })?;
        let signature_fingerprint = Fingerprint::from_str(parts[3]).map_err(|e| {
            Self::Err::client_internal_error(format!(
                "failed to parse signature fingerprint from consensus state vote summary: {}",
                e
            ))
        })?;
        let timestamp = Time::parse_from_rfc3339(parts[5]).map_err(|e| {
            Self::Err::client_internal_error(format!(
                "failed to parse timestamp from consensus state vote summary: {}",
                e
            ))
        })?;

        Ok(Self {
            validator_index,
            validator_address_fingerprint,
            height,
            round,
            vote_type,
            block_id_hash_fingerprint,
            signature_fingerprint,
            timestamp,
        })
    }
}

impl fmt::Display for VoteSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Vote{{{}:{} {}/{:02}/{}({}) {} {} @ {}}}",
            self.validator_index,
            self.validator_address_fingerprint,
            self.height,
            self.round.value(),
            i32::from(self.vote_type),
            self.vote_type,
            self.block_id_hash_fingerprint,
            self.signature_fingerprint,
            self.timestamp.to_rfc3339(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fingerprint(Vec<u8>);

impl FromStr for Fingerprint {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(hex::decode_upper(s).map_err(|e| {
            Self::Err::client_internal_error(format!(
                "failed to parse fingerprint as an uppercase hexadecimal string: {}",
                e
            ))
        })?))
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_bytes = hex::encode_upper(&self.0);
        let hex_string = String::from_utf8(hex_bytes).unwrap();
        write!(f, "{}", hex_string)
    }
}

impl AsRef<[u8]> for Fingerprint {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: account::Id,
    pub index: i32,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_vote_summary() {
        let src = "Vote{0:000001E443FD 1262197/00/1(Prevote) 634ADAF1F402 7BB974E1BA40 @ 2019-08-01T11:52:35.513572509Z}";
        let summary = VoteSummary::from_str(src).unwrap();
        assert_eq!(summary.validator_index, 0);
        assert_eq!(
            summary.validator_address_fingerprint.0,
            vec![0, 0, 1, 228, 67, 253]
        );
        assert_eq!(summary.height.value(), 1262197);
        assert_eq!(summary.round.value(), 0);
        assert_eq!(summary.vote_type, vote::Type::Prevote);
        assert_eq!(
            summary.block_id_hash_fingerprint.0,
            vec![99, 74, 218, 241, 244, 2]
        );
        assert_eq!(
            summary.signature_fingerprint.0,
            vec![123, 185, 116, 225, 186, 64]
        );
        assert_eq!(
            summary.timestamp.to_rfc3339(),
            "2019-08-01T11:52:35.513572509Z"
        );
    }

    #[test]
    fn serialize_vote_summary() {
        let summary = VoteSummary {
            validator_index: 0,
            validator_address_fingerprint: Fingerprint(vec![0, 0, 1, 228, 67, 253]),
            height: Height::from(1262197_u32),
            round: Round::from(0_u8),
            vote_type: vote::Type::Prevote,
            block_id_hash_fingerprint: Fingerprint(vec![99, 74, 218, 241, 244, 2]),
            signature_fingerprint: Fingerprint(vec![123, 185, 116, 225, 186, 64]),
            timestamp: Time::parse_from_rfc3339("2019-08-01T11:52:35.513572509Z").unwrap(),
        };
        assert_eq!(summary.to_string(), "Vote{0:000001E443FD 1262197/00/1(Prevote) 634ADAF1F402 7BB974E1BA40 @ 2019-08-01T11:52:35.513572509Z}");
    }
}
