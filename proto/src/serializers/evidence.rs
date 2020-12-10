use crate::tendermint::types::evidence::Sum;
use crate::tendermint::types::Evidence;

/// EvidenceVariant helper struct for evidence serialization
/// This is a workaround until we figure a better way of JSON serializing evidence.
/// It is a modified copy of the crate::tendermint::types::evidence::Sum struct.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
#[serde(tag = "type", content = "value")]
pub enum EvidenceVariant {
    #[serde(rename = "tendermint/DuplicateVoteEvidence")]
    DuplicateVoteEvidence(crate::tendermint::types::DuplicateVoteEvidence),
    #[serde(rename = "tendermint/LightClientAttackEvidence")]
    LightClientAttackEvidence(crate::tendermint::types::LightClientAttackEvidence),
}

impl From<EvidenceVariant> for Evidence {
    fn from(value: EvidenceVariant) -> Self {
        match value {
            EvidenceVariant::DuplicateVoteEvidence(d) => Evidence {
                sum: Some(Sum::DuplicateVoteEvidence(d)),
            },
            EvidenceVariant::LightClientAttackEvidence(l) => Evidence {
                sum: Some(Sum::LightClientAttackEvidence(l)),
            },
        }
    }
}

impl From<Evidence> for EvidenceVariant {
    fn from(value: Evidence) -> Self {
        let sum = value.sum.unwrap(); // Todo: Error handling
        match sum {
            Sum::DuplicateVoteEvidence(d) => Self::DuplicateVoteEvidence(d),
            Sum::LightClientAttackEvidence(l) => Self::LightClientAttackEvidence(l),
        }
    }
}

impl From<Sum> for EvidenceVariant {
    fn from(_: Sum) -> Self {
        unimplemented!() // Prost adds extra annotations on top of Sum that are not used.
    }
}

impl From<EvidenceVariant> for Sum {
    fn from(_: EvidenceVariant) -> Self {
        unimplemented!() // Prost adds extra annotations on top of Sum that are not used.
    }
}
