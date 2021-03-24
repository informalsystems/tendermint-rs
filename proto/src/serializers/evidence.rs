use crate::tendermint::types::evidence::Sum;
use crate::tendermint::types::Evidence;

/// EvidenceVariant helper struct for evidence serialization
/// This is a workaround until we figure a better way of JSON serializing evidence.
/// It is a modified copy of the crate::tendermint::types::evidence::Sum struct.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
#[serde(tag = "type", content = "value")]
pub enum EvidenceVariant {
    /// Provided for when the evidence struct's optional `sum` field is `None`.
    None,
    #[serde(rename = "tendermint/DuplicateVoteEvidence")]
    DuplicateVoteEvidence(crate::tendermint::types::DuplicateVoteEvidence),
    #[serde(rename = "tendermint/LightClientAttackEvidence")]
    LightClientAttackEvidence(crate::tendermint::types::LightClientAttackEvidence),
}

impl From<EvidenceVariant> for Evidence {
    fn from(value: EvidenceVariant) -> Self {
        match value {
            EvidenceVariant::None => Evidence { sum: None },
            _ => Evidence {
                sum: Some(value.into()),
            },
        }
    }
}

impl From<Evidence> for EvidenceVariant {
    fn from(value: Evidence) -> Self {
        match value.sum {
            Some(sum) => sum.into(),
            None => Self::None,
        }
    }
}

impl From<Sum> for EvidenceVariant {
    fn from(value: Sum) -> Self {
        match value {
            Sum::DuplicateVoteEvidence(d) => Self::DuplicateVoteEvidence(d),
            Sum::LightClientAttackEvidence(l) => Self::LightClientAttackEvidence(l),
        }
    }
}

impl From<EvidenceVariant> for Sum {
    fn from(value: EvidenceVariant) -> Self {
        match value {
            // This should never be called - should be handled instead in the
            // `impl From<EvidenceVariant> for Evidence` above.
            EvidenceVariant::None => {
                panic!("non-existent evidence cannot be converted into its protobuf representation")
            }
            EvidenceVariant::DuplicateVoteEvidence(d) => Self::DuplicateVoteEvidence(d),
            EvidenceVariant::LightClientAttackEvidence(l) => Self::LightClientAttackEvidence(l),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Minimally reproduce https://github.com/informalsystems/tendermint-rs/issues/782
    #[test]
    fn empty_evidence() {
        let ev = Evidence { sum: None };
        let ev_json = serde_json::to_string(&ev).unwrap();
        let ev_deserialized = serde_json::from_str::<Evidence>(&ev_json).unwrap();
        assert_eq!(ev, ev_deserialized);
    }
}
