//! Encoding/decoding mechanisms for ABCI requests and responses.
//!
//! Implements the [Tendermint Socket Protocol][tsp].
//!
//! [tsp]: https://docs.tendermint.com/master/spec/abci/client-server.html#tsp

use bytes::{Buf, BufMut, BytesMut};
use prost::Message;
use std::io::{Read, Write};
use std::marker::PhantomData;
use tendermint_proto::abci::{Request, Response};

use crate::error::{self, Error};

/// The maximum number of bytes we expect in a varint. We use this to check if
/// we're encountering a decoding error for a varint.
pub const MAX_VARINT_LENGTH: usize = 16;

/// The server receives incoming requests, and sends outgoing responses.
pub type ServerCodec<S> = Codec<S, Request, Response>;

#[cfg(feature = "client")]
/// The client sends outgoing requests, and receives incoming responses.
pub type ClientCodec<S> = Codec<S, Response, Request>;

/// Allows for iteration over `S` to produce instances of `I`, as well as
/// sending instances of `O`.
pub struct Codec<S, I, O> {
    stream: S,
    // Long-running read buffer
    read_buf: BytesMut,
    // Fixed-length read window
    read_window: Vec<u8>,
    write_buf: BytesMut,
    _incoming: PhantomData<I>,
    _outgoing: PhantomData<O>,
}

impl<S, I, O> Codec<S, I, O>
where
    S: Read + Write,
    I: Message + Default,
    O: Message,
{
    /// Constructor.
    pub fn new(stream: S, read_buf_size: usize) -> Self {
        Self {
            stream,
            read_buf: BytesMut::new(),
            read_window: vec![0_u8; read_buf_size],
            write_buf: BytesMut::new(),
            _incoming: Default::default(),
            _outgoing: Default::default(),
        }
    }
}

// Iterating over a codec produces instances of `Result<I>`.
impl<S, I, O> Iterator for Codec<S, I, O>
where
    S: Read,
    I: Message + Default,
{
    type Item = Result<I, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to decode an incoming message from our buffer first
            match decode_length_delimited::<I>(&mut self.read_buf) {
                Ok(Some(incoming)) => return Some(Ok(incoming)),
                Err(e) => return Some(Err(e)),
                _ => (), // not enough data to decode a message, let's continue.
            }

            // If we don't have enough data to decode a message, try to read
            // more
            let bytes_read = match self.stream.read(self.read_window.as_mut()) {
                Ok(br) => br,
                Err(e) => return Some(Err(error::io_error(e))),
            };
            if bytes_read == 0 {
                // The underlying stream terminated
                return None;
            }
            self.read_buf
                .extend_from_slice(&self.read_window[..bytes_read]);
        }
    }
}

impl<S, I, O> Codec<S, I, O>
where
    S: Write,
    O: Message,
{
    /// Send a message using this codec.
    pub fn send(&mut self, message: O) -> Result<(), Error> {
        encode_length_delimited(message, &mut self.write_buf)?;
        while !self.write_buf.is_empty() {
            let bytes_written = self
                .stream
                .write(self.write_buf.as_ref())
                .map_err(error::io_error)?;

            if bytes_written == 0 {
                return Err(error::io_error(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "failed to write to underlying stream",
                )));
            }
            self.write_buf.advance(bytes_written);
        }

        self.stream.flush().map_err(error::io_error)?;

        Ok(())
    }
}

/// Encode the given message with a length prefix.
pub fn encode_length_delimited<M, B>(message: M, mut dst: &mut B) -> Result<(), Error>
where
    M: Message,
    B: BufMut,
{
    let mut buf = BytesMut::new();
    message.encode(&mut buf).map_err(error::encode_error)?;

    let buf = buf.freeze();
    encode_varint(buf.len() as u64, &mut dst);
    dst.put(buf);
    Ok(())
}

/// Attempt to decode a message of type `M` from the given source buffer.
pub fn decode_length_delimited<M>(src: &mut BytesMut) -> Result<Option<M>, Error>
where
    M: Message + Default,
{
    let src_len = src.len();
    let mut tmp = src.clone().freeze();
    let encoded_len = match decode_varint(&mut tmp) {
        Ok(len) => len,
        // We've potentially only received a partial length delimiter
        Err(_) if src_len <= MAX_VARINT_LENGTH => return Ok(None),
        Err(e) => return Err(e),
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
        let res = M::decode(&mut result_bytes).map_err(error::decode_error)?;

        Ok(Some(res))
    }
}

// encode_varint and decode_varint will be removed once
// https://github.com/tendermint/tendermint/issues/5783 lands in Tendermint.
pub fn encode_varint<B: BufMut>(val: u64, mut buf: &mut B) {
    prost::encoding::encode_varint(val << 1, &mut buf);
}

pub fn decode_varint<B: Buf>(mut buf: &mut B) -> Result<u64, Error> {
    let len = prost::encoding::decode_varint(&mut buf).map_err(error::decode_error)?;
    Ok(len >> 1)
}
