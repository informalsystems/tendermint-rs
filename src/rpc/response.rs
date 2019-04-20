//! JSONRPC response types

// TODO(tarcieri): remove this when functions below are all used
#![allow(dead_code)]

use failure::{format_err, Error};
use serde::{
    de::{DeserializeOwned, Error as DeError},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    fmt::{self, Display},
    time::Duration,
};

/// JSONRPC responses
pub trait Response: Serialize + DeserializeOwned + Sized {
    /// Parse a JSONRPC response from a JSON string
    fn from_json(response: &str) -> Result<Self, Error> {
        let wrapper: ResponseWrapper<Self> =
            serde_json::from_str(response).map_err(|e| format_err!("error parsing JSON: {}", e))?;

        // TODO(tarcieri): check JSONRPC version/ID?
        Ok(wrapper.result)
    }
}

/// JSONRPC response wrapper (i.e. message envelope)
#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseWrapper<R> {
    /// JSONRPC version
    pub jsonrpc: Version,

    /// ID
    pub id: Id,

    /// Result
    pub result: R,
}

/// JSONRPC version
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Version(String);

impl Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// JSONRPC ID
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Id(String);

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// Parse `u64` from a JSON string
pub(crate) fn parse_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse::<u64>()
        .map_err(|e| D::Error::custom(format!("{}", e)))
}

/// Serialize `u64` as a JSON string
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn serialize_u64<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    format!("{}", value).serialize(serializer)
}

/// Parse `Duration` from a JSON string containing a nanosecond count
fn parse_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    // TODO(tarcieri): handle 64-bit overflow?
    let nanos = String::deserialize(deserializer)?
        .parse::<u64>()
        .map_err(|e| D::Error::custom(format!("{}", e)))?;

    Ok(Duration::from_nanos(nanos))
}

/// Serialize `Duration` as a JSON string containing a nanosecond count
fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // TODO(tarcieri): use `as_nanos` when we're Rust 1.33+
    format!(
        "{}",
        (duration.as_secs() * 1_000_000_000) + duration.subsec_nanos() as u64
    )
    .serialize(serializer)
}
