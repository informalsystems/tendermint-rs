//! Serde JSON serializers
//!
//! Serializers and deserializers for a transparent developer experience.
//!
//! CAUTION: There are no guarantees for backwards compatibility, this module should be considered
//! an internal implementation detail which can vanish without further warning. Use at your own
//! risk.
//!
//! All serializers are presented in a serializers::<Rust_nickname>::<JSON_representation_name>
//! format.
//!
//! This example shows how to serialize Vec<u8> into different types of strings:
//! ```ignore
//! use serde::{Serialize, Deserialize};
//! use tendermint_proto::serializers;
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

// Todo: remove dead_code allowance as soon as more types are implemented
#![allow(dead_code)]
pub mod bytes;
pub mod from_str;
pub mod time_duration;
pub mod option_timestamp;
pub mod txs;
pub mod nullable;

mod custom;
pub use custom::null_as_default;
