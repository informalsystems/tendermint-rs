//! Tags

use core::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use tendermint_proto::serializers::bytes::base64string;

use crate::{error::Error, prelude::*};

/// Tags
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tag {
    /// Key
    pub key: Key,

    /// Value
    pub value: Value,
}

/// Tag keys
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct Key(
    #[serde(
        serialize_with = "base64string::serialize",
        deserialize_with = "base64string::deserialize_to_string"
    )]
    String,
);

impl Key {
    pub fn new(key: String) -> Self {
        Self(key)
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<String> for Key {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl FromStr for Key {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// Tag values
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Value(
    #[serde(
        serialize_with = "base64string::serialize",
        deserialize_with = "base64string::deserialize_to_string"
    )]
    String,
);

impl Value {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for Value {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl FromStr for Value {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tag_serde() {
        let json = r#"{"key": "cGFja2V0X3RpbWVvdXRfaGVpZ2h0", "value": "MC00ODQw"}"#;
        let tag: Tag = serde_json::from_str(json).unwrap();
        assert_eq!("packet_timeout_height", tag.key.0);
        assert_eq!("0-4840", tag.value.0);
    }
}
