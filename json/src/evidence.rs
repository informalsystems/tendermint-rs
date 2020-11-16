//! Evidence-related data structures for Tendermint.

use crate::block::SignedHeader;
use crate::serializers;
use crate::vote::Vote;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceData {
    #[serde(with = "serializers::nullable")]
    pub evidence: Vec<Evidence>,
    #[serde(default)]
    pub hash: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Evidence {
    /// Duplicate vote evidence
    #[serde(rename = "tendermint/DuplicateVoteEvidence")]
    DuplicateVote(DuplicateVoteEvidence),

    /// Conflicting headers evidence - Todo: this is not implemented in protobuf, it's ignored now
    #[serde(rename = "tendermint/ConflictingHeadersEvidence")]
    ConflictingHeaders(Box<ConflictingHeadersEvidence>),

    /// LightClient attack evidence - Todo: Implement details
    LightClientAttackEvidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DuplicateVoteEvidence {
    pub vote_a: Vote,
    pub vote_b: Vote,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConflictingHeadersEvidence {
    #[serde(rename = "H1")]
    pub h1: SignedHeader,
    #[serde(rename = "H2")]
    pub h2: SignedHeader,
}
