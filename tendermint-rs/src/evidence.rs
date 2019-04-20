//! Evidence of Byzantine behavior

use std::slice;
#[cfg(feature = "serde")]
use {
    serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer},
    subtle_encoding::base64,
};

/// Evidence data
// TODO(tarcieri): parse evidence (amino?)
#[derive(Clone, Debug)]
pub struct Evidence(Vec<u8>);

impl Evidence {
    /// Create a new raw evidence value from a byte vector
    pub fn new<V>(into_vec: V) -> Evidence
    where
        V: Into<Vec<u8>>,
    {
        // TODO(tarcieri): parse/validate evidence contents
        Evidence(into_vec.into())
    }

    /// Serialize this evidence as a bytestring
    pub fn to_bytes(&self) -> Vec<u8> {
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
        String::from_utf8(base64::encode(self.to_bytes()))
            .unwrap()
            .serialize(serializer)
    }
}

/// Evidence collection
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, Debug)]
pub struct Collection {
    evidence: Option<Vec<Evidence>>,
}

impl Collection {
    /// Create a new evidence collection
    pub fn new<I>(into_evidence: I) -> Collection
    where
        I: Into<Vec<Evidence>>,
    {
        Collection {
            evidence: Some(into_evidence.into()),
        }
    }

    /// Convert this collection into a vector
    pub fn into_vec(self) -> Vec<Evidence> {
        self.evidence.unwrap_or_else(|| vec![])
    }

    /// Iterate over the evidence in the collection
    pub fn iter(&self) -> slice::Iter<Evidence> {
        self.as_ref().iter()
    }
}

impl AsRef<[Evidence]> for Collection {
    fn as_ref(&self) -> &[Evidence] {
        self.evidence
            .as_ref()
            .map(|evidence| evidence.as_slice())
            .unwrap_or_else(|| &[])
    }
}
