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
    /// An error will be returned if the buffer does not have sufficient capacity.
    fn encode_length_delimited<B: BufMut>(self, buf: &mut B) -> Result<(), Error>;

    /// Decodes an instance of the message from a buffer and then converts it into DomainType.
    ///
    /// The entire buffer will be consumed.
    fn decode<B: Buf>(buf: B) -> Result<Self, Error>;

    /// Decodes a length-delimited instance of the message from the buffer.
    fn decode_length_delimited<B: Buf>(buf: B) -> Result<Self, Error>;

    /// Returns the encoded length of the message without a length delimiter.
    fn encoded_len(self) -> usize;
}
