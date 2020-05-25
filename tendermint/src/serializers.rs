//! Serde serializers
//!
//! Serializers and deserializers for a transparent developer experience.
//!
//! All serializers are presented in a serializers::<Rust_nickname>::<JSON_representation_name>
//! format.
//!
//! This example shows how to serialize Vec<u8> into different types of strings:
//! ```ignore
//! use serde::{Serialize, Deserialize};
//! use serializers;
//!
//! #[derive(Serialize, Deserialize)]
//! struct ByteTypes {
//!
//!     #[serde(with="serializers::bytes::hexstring")]
//!     hexbytes: Vec<u8>,
//!
//!     #[serde(with="serializers::bytes::base64string")]
//!     base64bytes: Vec<u8>,
//!
//!     #[serde(with="serializers::bytes::string")]
//!     bytes: Vec<u8>,
//!
//! }
//! ```
//!
//! Available serializers:
//! i64                  <-> string:               #[serde(with="serializers::from_str")]
//! u64                  <-> string:               #[serde(with="serializers::from_str")]
//! std::time::Duration  <-> nanoseconds as string #[serde(with="serializers::time_duration")]
//! Vec<u8>              <-> HexString:            #[serde(with="serializers::bytes::hexstring")]
//! Vec<u8>              <-> Base64String:         #[serde(with="serializers::bytes::base64string")]
//! Vec<u8>              <-> String:               #[serde(with="serializers::bytes::string")]
//!
//! Notes:
//! * Any type that has the "FromStr" trait can be serialized into a string with
//!   serializers::primitives::string.
//! * serializers::bytes::* deserializes a null value into an empty vec![].

pub(crate) mod bytes;
pub(crate) mod from_str;
pub(crate) mod time_duration;

#[cfg(test)]
mod tests;

mod custom;
pub(crate) use custom::parse_non_empty_block_id;
pub(crate) use custom::parse_non_empty_hash;
pub(crate) use custom::parse_non_empty_id;
pub(crate) use custom::parse_non_empty_signature;
