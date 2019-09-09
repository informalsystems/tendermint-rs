//! ABCI Merkle proofs

use crate::error::Error;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Display},
    str::FromStr,
};
use subtle_encoding::{Encoding, Hex};

/// ABCI Merkle proofs
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Proof(Vec<u8>);

impl AsRef<[u8]> for Proof {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Display for Proof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            &Hex::upper_case().encode_to_string(&self.0).unwrap()
        )
    }
}

impl FromStr for Proof {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let bytes = Hex::upper_case().decode(s)?;
        Ok(Proof(bytes))
    }
}

impl<'de> Deserialize<'de> for Proof {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;
        Ok(Self::from_str(&hex).map_err(|e| D::Error::custom(format!("{}", e)))?)
    }
}

impl Serialize for Proof {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
