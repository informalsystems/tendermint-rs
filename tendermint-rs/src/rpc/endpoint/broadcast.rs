//! `/broadcast_tx_*` endpoint JSONRPC wrappers

pub mod tx_async;
pub mod tx_commit;
pub mod tx_sync;

use crate::Error;
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Display},
    str::FromStr,
};
use subtle_encoding::hex;

/// Transaction broadcast response codes.
///
/// These presently use 0 for success and non-zero for errors, however there
/// is ample discussion about potentially supporting non-zero success cases
/// as well.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum Code {
    /// Success
    Ok,

    /// Error codes
    Err(u32),
}

impl Code {
    /// Was the response OK?
    pub fn is_ok(self) -> bool {
        match self {
            Code::Ok => true,
            Code::Err(_) => false,
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
    fn from(value: u32) -> Code {
        match value {
            0 => Code::Ok,
            err => Code::Err(err),
        }
    }
}

impl From<Code> for u32 {
    fn from(code: Code) -> u32 {
        match code {
            Code::Ok => 0,
            Code::Err(err) => err,
        }
    }
}

impl<'de> Deserialize<'de> for Code {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Code::from(
            String::deserialize(deserializer)?
                .parse::<u32>()
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        ))
    }
}

impl Serialize for Code {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value().serialize(serializer)
    }
}

/// Transaction data
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Data(Vec<u8>);

impl Data {
    /// Borrow the data as bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for Data {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

impl FromStr for Data {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept either upper or lower case hex
        let bytes = hex::decode_upper(s)
            .or_else(|_| hex::decode(s))
            .map_err(|_| Error::Parse)?;

        Ok(Data(bytes))
    }
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = hex::decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(|e| D::Error::custom(format!("{}", e)))?;

        Ok(Self(bytes))
    }
}

impl Serialize for Data {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

/// Transaction log
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Log(String);

impl Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
