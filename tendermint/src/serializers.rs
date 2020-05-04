//! Serde serializers
//!
//! Serializers and deserializers for a transparent developer experience.
//!
//! Serializers for the 'with' annotation are presented in a
//! serializers::<rust_nickname>::<json_representation_name> format.
//!
//! Serializers for the 'try_from' annotation are presented in a
//! serializers::Raw<CustomType> format.
//!
//! This example shows how to serialize Vec<u8> into different types of strings:
//! ```
//! use serde::{Serialize, Deserialize};
//! use tendermint::serializers;
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
//! Available serializer modules for the 'with' annotation:
//! i64                  <-> string                #[serde(with="serializers::primitives::string")]
//! u64                  <-> string                #[serde(with="serializers::primitives::string")]
//! std::time::Duration  <-> nanoseconds as string #[serde(with="serializers::timeduration::string")]
//! Vec<u8>              <-> HexString             #[serde(with="serializers::bytes::hexstring")]
//! Vec<u8>              <-> Base64String          #[serde(with="serializers::bytes::base64string")]
//! Vec<u8>              <-> string                #[serde(with="serializers::bytes::string")]
//!
//! Notes:
//! * Any type that has the "FromStr" trait can be serialized into a string with serializers::primitives::string.
//! * serializers::bytes::* deserializes a null value into an empty vec![].
//!
//! Available serializer types for the 'try_from' annotation:
//! block::CommitSig     <-> RawCommitSig          #[serde(try_from="serializers::RawCommitSig")]
//!
//! Notes:
//! * The Raw... types are implemented based on the Tendermint specifications
//!

pub mod bytes;
pub mod primitives;
pub mod timeduration;

mod raw_commit_sig;
pub use raw_commit_sig::BlockIDFlag;
pub use raw_commit_sig::RawCommitSig;

/// Todo: refactor and remove
mod other;
pub use other::parse_non_empty_block_id;
pub use other::parse_non_empty_hash;
