//! ABCI application response codes.
//!
//! <https://tendermint.com/docs/spec/abci/abci.html#errors>

use serde::de::{Deserialize, Deserializer, Visitor};
use serde::{Serialize, Serializer};
use std::convert::TryFrom;
use std::fmt;

/// ABCI application response codes.
///
/// These presently use 0 for success and non-zero for errors:
///
/// <https://tendermint.com/docs/spec/abci/abci.html#errors>
///
/// Note that in the future there may potentially be non-zero success codes.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Code {
    /// Success
    Ok,

    /// Error codes
    Err(u32),
}

impl Default for Code {
    fn default() -> Self {
        Self::Ok
    }
}

impl Code {
    /// Was the response OK?
    pub fn is_ok(self) -> bool {
        match self {
            Self::Ok => true,
            Self::Err(_) => false,
        }
    }

    /// Was the response an error?
    pub fn is_err(self) -> bool {
        !self.is_ok()
    }

    /// Get the integer error value for this code
    pub fn value(self) -> u32 {
        u32::from(self)
    }
}

impl From<u32> for Code {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Ok,
            err => Self::Err(err),
        }
    }
}

impl From<u64> for Code {
    fn from(value: u64) -> Self {
        let code = u32::try_from(value).expect("code out of range");
        Self::from(code)
    }
}

impl From<Code> for u32 {
    fn from(code: Code) -> Self {
        match code {
            Code::Ok => 0,
            Code::Err(err) => err,
        }
    }
}

impl Serialize for Code {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Code {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CodeVisitor;

        impl<'de> Visitor<'de> for CodeVisitor {
            type Value = Code;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt.write_str("integer or string")
            }

            fn visit_u64<E>(self, val: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Code::from(val))
            }

            fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match val.parse::<u64>() {
                    Ok(val) => self.visit_u64(val),
                    Err(_) => Err(E::custom("failed to parse integer")),
                }
            }
        }

        deserializer.deserialize_any(CodeVisitor)
    }
}
