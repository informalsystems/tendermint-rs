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
//! use crate::serializers;
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
//!
//! | Field Type   | String Format                 | Serializer        |
//! |--------------|-------------------------------|-------------------|
//! | `i64`        | e.g. `-5`                     | [`from_str`]      |
//! | `u64`        | e.g. `100`                    | [`from_str`]      |
//! | [`Duration`] | Nanoseconds (e.g. `100`)      | [`time_duration`] |
//! | `Vec<u8>`    | Hexadecimal (e.g. `1AF2B3C4`) | [`hexstring`]     |
//! | `Vec<u8>`    | Base64-encoded                | [`base64string`]  |
//! | `Vec<u8>`    | Raw bytes in string           | [`string`]        |
//!
//! Notes:
//! * Any type that has the "FromStr" trait can be serialized into a string with
//!   serializers::primitives::string.
//! * serializers::bytes::* deserializes a null value into an empty vec![].
//!
//! [`Duration`]: core::time::Duration
//! [`hexstring`]: bytes::hexstring
//! [`base64string`]: bytes::base64string
//! [`string`]: bytes::string

// Todo: remove dead_code allowance as soon as more types are implemented
#![allow(dead_code)]
pub mod bytes;
pub mod evidence;
pub mod from_str;
pub mod nullable;
pub mod optional;
pub mod optional_from_str;
pub mod part_set_header_total;
pub mod time_duration;
pub mod timestamp;
pub mod txs;
