//! DomainType trait
//!
//! The DomainType trait allows separation of the data sent on the wire (currently encoded using
//! protobuf) from the structures used in Rust. The structures used to encode/decode from/to the
//! wire are called "Raw" types (they mirror the definitions in the specifications) and the Rust
//! types we use internally are called the "Domain" types. These Domain types can implement
//! additional checks and conversions to consume the incoming data easier for a Rust developer.
//!
//! The benefits include decoding the wire into a struct that is inherently valid as well as hiding
//! the encoding and decoding details from the developer. This latter is important if/when we decide
//! to exchange the underlying Prost library with something else. (Another protobuf implementation
//! or a completely different encoding.) Encoding is not the core product it's a necessary
//! dependency.
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
//! Requirements:
//! * The DomainType trait requires the struct to implement the Clone trait.
//! * Any RawType structure implements the prost::Message trait. (protobuf struct)
//! * The DomainType trait requires that the TryFrom<RawType> implemented on the structure has an
//!   error type that implements Into<BoxError>. (The current implementations with anomaly are
//!   fine.)
//!
//! How to implement a DomainType struct:
//! 1. Implement your struct based on your expectations for the developer
//! 2. Add `impl DomainType<MyRawType> for MyDomainType {}` blanket implementation of the trait
//! 4. Implement the `TryFrom<MyRawType> for MyDomainType` trait
//! 5. Implement the `From<MyDomainType> for MyRawType` trait

use crate::{Error, Kind};
use anomaly::BoxError;
use bytes::{Buf, BufMut};
use prost::{encoding::encoded_len_varint, Message};
use std::convert::{TryFrom, TryInto};

/// DomainType trait allows protobuf encoding and decoding for domain types
pub trait DomainType<T: Message + From<Self> + Default>
where
    Self: Sized + Clone + TryFrom<T>,
    <Self as TryFrom<T>>::Error: Into<BoxError>,
{
    /// Encodes the DomainType into a buffer.
    ///
    /// This function replaces the Prost::Message encode() function for DomainTypes.
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), Error> {
        T::from(self.clone())
            .encode(buf)
            .map_err(|e| Kind::EncodeMessage.context(e).into())
    }

    /// Encodes the DomainType with a length-delimiter to a buffer.
    ///
    /// An error will be returned if the buffer does not have sufficient capacity.
    ///
    /// This function replaces the Prost::Message encode_length_delimited() function for
    /// DomainTypes.
    fn encode_length_delimited<B: BufMut>(&self, buf: &mut B) -> Result<(), Error> {
        T::from(self.clone())
            .encode_length_delimited(buf)
            .map_err(|e| Kind::EncodeMessage.context(e).into())
    }

    /// Decodes an instance of the message from a buffer and then converts it into DomainType.
    ///
    /// The entire buffer will be consumed.
    ///
    /// This function replaces the Prost::Message decode() function for DomainTypes.
    fn decode<B: Buf>(buf: B) -> Result<Self, Error> {
        T::decode(buf).map_or_else(
            |e| Err(Kind::DecodeMessage.context(e).into()),
            |t| Self::try_from(t).map_err(|e| Kind::TryIntoDomainType.context(e).into()),
        )
    }

    /// Decodes a length-delimited instance of the message from the buffer.
    ///
    /// The entire buffer will be consumed.
    ///
    /// This function replaces the Prost::Message decode_length_delimited() function for
    /// DomainTypes.
    fn decode_length_delimited<B: Buf>(buf: B) -> Result<Self, Error> {
        T::decode_length_delimited(buf).map_or_else(
            |e| Err(Kind::DecodeMessage.context(e).into()),
            |t| Self::try_from(t).map_err(|e| Kind::TryIntoDomainType.context(e).into()),
        )
    }

    /// Returns the encoded length of the message without a length delimiter.
    ///
    /// This function replaces the Prost::Message encoded_len() function for DomainTypes.
    fn encoded_len(&self) -> usize {
        T::from(self.clone()).encoded_len()
    }

    /// Encodes the DomainType into a protobuf-encoded Vec<u8>
    fn encode_vec(&self) -> Result<Vec<u8>, Error> {
        let mut wire = Vec::with_capacity(self.encoded_len());
        self.encode(&mut wire).map(|_| wire)
    }

    /// Decodes a protobuf-encoded instance of the message from a Vec<u8> and then converts it into
    /// DomainType.
    fn decode_vec(v: &[u8]) -> Result<Self, Error> {
        Self::decode(v)
    }

    /// Encodes the DomainType with a length-delimiter to a Vec<u8> protobuf-encoded message.
    fn encode_length_delimited_vec(&self) -> Result<Vec<u8>, Error> {
        let len = self.encoded_len();
        let lenu64 = len.try_into().map_err(|e| Kind::EncodeMessage.context(e))?;
        let mut wire = Vec::with_capacity(len + encoded_len_varint(lenu64));
        self.encode_length_delimited(&mut wire).map(|_| wire)
    }

    /// Decodes a protobuf-encoded instance of the message with a length-delimiter from a Vec<u8>
    /// and then converts it into DomainType.
    fn decode_length_delimited_vec(v: &[u8]) -> Result<Self, Error> {
        Self::decode_length_delimited(v)
    }
}
