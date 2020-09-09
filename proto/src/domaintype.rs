//! DomainType trait
//!
//! The DomainType trait allows separation of the data sent on the wire (currently encoded using
//! protobuf) from the structures used in Rust. The structures used to encode/decode from/to the wire
//! are called "Raw" types (they mirror the definitions in the specifications) and the Rust types
//! we use internally are called the "Domain" types. These Domain types can implement additional
//! checks and conversions to consume the incoming data easier for a Rust developer.
//!
//! The benefits include decoding the wire into a struct that is inherently valid as well as hiding
//! the encoding and decoding details from the developer. This latter is important if/when we decide
//! to exchange the underlying Prost library with something else. (Another protobuf implementation
//! or a completely different encoding.) Encoding is not the core product of Tendermint it's a
//! necessary dependency.
//!
//!
//! Decode: bytestream -> Raw -> Domain
//! The `decode` function takes two steps to decode from a bytestream to a DomainType:
//!
//! 1. Decode the bytestream into a Raw type using the Prost library,
//! 2. Transform that Raw type into a Domain type using the TryFrom trait of the DomainType.
//!
//!
//! Encode: Domain -> Raw -> bytestream
//! The `encode` function takes two steps to encode a DomainType into a bytestream:
//!
//! 1. Transform the Domain type into a Raw type using the From trait of the DomainType,
//! 2. Encode the Raw type into a bytestream using the Prost library.
//!
//!
//! Note that in the case of encode, the transformation to Raw type is infallible:
//! Rust structs should always be ready to be encoded to the wire.
//!
//! Note that the Prost library and the TryFrom method have their own set of errors. These are
//! merged into a custom Error type defined in this crate for easier handling.
//!
//!
//! How to implement a DomainType struct:
//! 1. Implement your struct based on your expectations for the developer
//! 2. Add the derive macro `#[derive(DomainType)]` on top of it
//! 3. Add the Raw type as a parameter of the DomainType trait (`[rawtype(MyRawType)]`)
//! 4. Implement the `TryFrom<MyRawType> for MyDomainType` trait
//! 5. Implement the `From<MyDomainType> for MyRawType` trait
//!
//! Note: the `[rawtype()]` parameter is similar to how `serde` implements serialization through a
//! `[serde(with="")]` interim type.
//!

use crate::Error;
use bytes::{Buf, BufMut};
use prost::Message;

/// DomainType trait allows protobuf encoding and decoding for domain types
pub trait DomainType<T: Message + From<Self>>: Sized {
    /// Encodes the DomainType into a buffer.
    ///
    /// The DomainType will be consumed.
    fn encode<B: BufMut>(self, buf: &mut B) -> Result<(), Error>;

    /// Encodes the DomainType with a length-delimiter to a buffer.
    ///
    /// The DomainType will be consumed.
    /// An error will be returned if the buffer does not have sufficient capacity.
    fn encode_length_delimited<B: BufMut>(self, buf: &mut B) -> Result<(), Error>;

    /// Decodes an instance of the message from a buffer and then converts it into DomainType.
    ///
    /// The entire buffer will be consumed.
    fn decode<B: Buf>(buf: B) -> Result<Self, Error>;

    /// Decodes a length-delimited instance of the message from the buffer.
    ///
    /// The entire buffer will be consumed.
    fn decode_length_delimited<B: Buf>(buf: B) -> Result<Self, Error>;

    /// Returns the encoded length of the message without a length delimiter.
    ///
    /// The DomainType will be consumed.
    fn encoded_len(self) -> usize;
}
