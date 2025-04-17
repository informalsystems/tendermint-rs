//! Combines [`from_str`] and [`allow_null`].
//!
//! Use this module to serialize and deserialize any `T` where `T` implements
//! [`FromStr`] and [`Display`] to convert from or into a string.
//! The serialized form is that of `Option<String>`,
//! and a nil is deserialized to the `Default` value. For JSON, this means both
//! quoted string values and `null` are accepted. A value is always serialized
//! as `Some<String>`.
//! Note that this can be used for all primitive data types.
//!
//! [`from_str`]: super::from_str
//! [`allow_null`]: super::allow_null

use core::fmt::Display;
use core::str::FromStr;

use serde::{de::Error as _, Deserialize, Deserializer, Serializer};

use crate::prelude::*;
use crate::serializers::cow_str::CowStr;

/// Deserialize a nullable string into T
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Default,
    <T as FromStr>::Err: Display,
{
    match <Option<CowStr<'_>>>::deserialize(deserializer)? {
        Some(s) => s.parse::<T>().map_err(D::Error::custom),
        None => Ok(T::default()),
    }
}

/// Serialize from T into string
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Display,
{
    serializer.serialize_some(&value.to_string())
}
