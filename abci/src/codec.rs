//! ABCI codec.

#[cfg(feature = "blocking")]
pub mod blocking;
#[cfg(feature = "non-blocking")]
pub mod non_blocking;

use crate::{Error, Result};
use bytes::{Buf, BufMut, BytesMut};
use tendermint::abci::request::Request;
use tendermint::abci::response::Response;
use tendermint_proto::Protobuf;

/// The size of the server's read buffer for incoming messages.
pub const SERVER_READ_BUF_SIZE: usize = 1024 * 1024;

/// The size of the client's read buffer for incoming messages.
pub const CLIENT_READ_BUF_SIZE: usize = 1024;

// The maximum number of bytes we expect in a varint. We use this to check if
// we're encountering a decoding error for a varint.
const MAX_VARINT_LENGTH: usize = 16;

/// A stateless encoder of `T` into its wire-level representation.
pub trait Encoder<T> {
    fn encode(value: T, dst: &mut BytesMut) -> Result<()>;
}

/// Encodes [`tendermint::abci::Request`]s into their wire-level
/// representation as per the Tendermint Socket Protocol.
pub struct RequestEncoder;

impl Encoder<Request> for RequestEncoder {
    fn encode(value: Request, mut dst: &mut BytesMut) -> Result<()> {
        encode_length_delimited(|mut b| Ok(value.encode(&mut b)?), &mut dst)
    }
}

/// Encodes [`tendermint::abci::Response`]s into their wire-level
/// representation as per the Tendermint Socket Protocol.
pub struct ResponseEncoder;

impl Encoder<Response> for ResponseEncoder {
    fn encode(value: Response, mut dst: &mut BytesMut) -> Result<()> {
        encode_length_delimited(|mut b| Ok(value.encode(&mut b)?), &mut dst)
    }
}

/// A potentially stateful decoder of `T` from its wire-level representation.
pub trait Decoder<T> {
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<T>>;
}

/// Decodes [`tendermint::abci::Request`]s from their wire-level
/// representation as per the Tendermint Socket Protocol.
pub struct RequestDecoder {
    read_buf: BytesMut,
}

impl RequestDecoder {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            read_buf: BytesMut::new(),
        }
    }
}

impl Decoder<Request> for RequestDecoder {
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Request>> {
        self.read_buf.put(buf);
        decode_length_delimited(&mut self.read_buf, |mut b| Ok(Request::decode(&mut b)?))
    }
}

impl Default for RequestDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Decodes [`tendermint::abci::Response`]s from their wire-level
/// representation as per the Tendermint Socket Protocol.
pub struct ResponseDecoder {
    read_buf: BytesMut,
}

impl ResponseDecoder {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            read_buf: BytesMut::new(),
        }
    }
}

impl Decoder<Response> for ResponseDecoder {
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Response>> {
        self.read_buf.put(buf);
        decode_length_delimited(&mut self.read_buf, |mut b| Ok(Response::decode(&mut b)?))
    }
}

impl Default for ResponseDecoder {
    fn default() -> Self {
        Self::new()
    }
}

// encode_varint and decode_varint will be removed once
// https://github.com/tendermint/tendermint/issues/5783 lands in Tendermint.
fn encode_varint<B: BufMut>(val: u64, mut buf: &mut B) {
    tendermint_proto::encode_varint(val << 1, &mut buf);
}

fn decode_varint<B: Buf>(mut buf: &mut B) -> Result<u64> {
    let len = tendermint_proto::decode_varint(&mut buf)
        .map_err(|_| Error::Protobuf(tendermint_proto::Kind::DecodeMessage.into()))?;
    Ok(len >> 1)
}

// Allows us to avoid having to re-export `prost::Message`.
// TODO(thane): Investigate a better approach here.
fn encode_length_delimited<F, B>(mut encode_fn: F, mut dst: &mut B) -> Result<()>
where
    F: FnMut(&mut BytesMut) -> Result<()>,
    B: BufMut,
{
    let mut buf = BytesMut::new();
    encode_fn(&mut buf)?;
    let buf = buf.freeze();
    encode_varint(buf.len() as u64, &mut dst);
    dst.put(buf);
    Ok(())
}

fn decode_length_delimited<F, T>(src: &mut BytesMut, mut decode_fn: F) -> Result<Option<T>>
where
    F: FnMut(&mut BytesMut) -> Result<T>,
{
    let src_len = src.len();
    let mut tmp = src.clone().freeze();
    let encoded_len = match decode_varint(&mut tmp) {
        Ok(len) => len,
        Err(e) => {
            return if src_len <= MAX_VARINT_LENGTH {
                // We've potentially only received a partial length delimiter
                Ok(None)
            } else {
                Err(e)
            };
        }
    };
    let remaining = tmp.remaining() as u64;
    if remaining < encoded_len {
        // We don't have enough data yet to decode the entire message
        Ok(None)
    } else {
        let delim_len = src_len - tmp.remaining();
        // We only advance the source buffer once we're sure we have enough
        // data to try to decode the result.
        src.advance(delim_len + (encoded_len as usize));

        let mut result_bytes = BytesMut::from(tmp.split_to(encoded_len as usize).as_ref());
        Ok(Some(decode_fn(&mut result_bytes)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tendermint::abci::request::Echo;

    #[test]
    fn single_request() {
        let request = Request::Echo(Echo {
            message: "Hello TSP!".to_owned(),
        });
        let mut buf = BytesMut::new();
        RequestEncoder::encode(request.clone(), &mut buf).unwrap();

        let mut decoder = RequestDecoder::new();
        let decoded_request = decoder.decode(&mut buf).unwrap().unwrap();

        assert_eq!(request, decoded_request);
    }

    #[test]
    fn multiple_requests() {
        let requests = (0..5)
            .map(|r| {
                Request::Echo(Echo {
                    message: format!("Request {}", r),
                })
            })
            .collect::<Vec<Request>>();
        let mut buf = BytesMut::new();
        requests
            .iter()
            .for_each(|request| RequestEncoder::encode(request.clone(), &mut buf).unwrap());

        let mut decoder = RequestDecoder::new();
        for request in requests {
            let decoded = decoder.decode(&mut buf).unwrap().unwrap();
            assert_eq!(decoded, request);
        }
    }
}
