//! Evidence of malfeasance by validators (i.e. signing conflicting votes).

use std::slice;
#[cfg(feature = "serde")]
use {
    crate::serializers,
    serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer},
    subtle_encoding::base64,
};

/// Evidence of malfeasance by validators (i.e. signing conflicting votes).
/// encoded using an Amino prefix. There is currently only a single type of
/// evidence: `DuplicateVoteEvidence`.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#evidence>
#[derive(Clone, Debug)]
pub struct Evidence(Vec<u8>);

impl Evidence {
    /// Create a new raw evidence value from a byte vector
    pub fn new<V>(into_vec: V) -> Evidence
    where
        V: Into<Vec<u8>>,
    {
        // TODO(tarcieri): parse/validate evidence contents from amino messages
        Evidence(into_vec.into())
    }

    /// Serialize this evidence as an Amino message bytestring
    pub fn to_amino_bytes(&self) -> Vec<u8> {
        self.0.clone()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Evidence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = base64::decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(|e| D::Error::custom(format!("{}", e)))?;

        Ok(Evidence::new(bytes))
    }
}

#[cfg(feature = "serde")]
impl Serialize for Evidence {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        String::from_utf8(base64::encode(self.to_amino_bytes()))
            .unwrap()
            .serialize(serializer)
    }
}

/// Evidence data is a wrapper for a list of `Evidence`.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#evidencedata>
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, Debug)]
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
    pub fn iter(&self) -> slice::Iter<Evidence> {
        self.as_ref().iter()
    }
}

impl AsRef<[Evidence]> for Data {
    fn as_ref(&self) -> &[Evidence] {
        self.evidence
            .as_ref()
            .map(Vec::as_slice)
            .unwrap_or_else(|| &[])
    }
}

/// Evidence collection parameters
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Params {
    /// Maximum allowed age for evidence to be collected
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serializers::serialize_u64",
            deserialize_with = "serializers::parse_u64"
        )
    )]
    pub max_age: u64,
}
