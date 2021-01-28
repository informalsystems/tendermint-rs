//! Blocking API for frame en/decoding.

use crate::codec::{
    Decoder, Encoder, RequestDecoder, RequestEncoder, ResponseDecoder, ResponseEncoder,
};
use crate::Result;
use bytes::{Buf, BytesMut};
use std::io::{Read, Write};
use std::marker::PhantomData;
use tendermint::abci::request::Request;
use tendermint::abci::response::Response;

/// An ABCI server decodes incoming [`tendermint::abci::request::Request`]s
/// and encodes outgoing [`tendermint::abci::response::Response`]s. The stream
/// `S` must implement [`std::io::Read`] and [`std::io::Write`].
pub type ServerCodec<S> = CodecImpl<ResponseEncoder, RequestDecoder, Response, Request, S>;

/// An ABCI client encodes outgoing [`tendermint::abci::request::Request`]s
/// and decodes incoming [`tendermint::abci::response::Response`]s. The stream
/// `S` must implement [`std::io::Read`] and [`std::io::Write`].
pub type ClientCodec<S> = CodecImpl<RequestEncoder, ResponseDecoder, Request, Response, S>;

/// A blocking codec that allows us to iterate over `S`, producing values of
/// type `Result<I>`, and send values of type `O`.
pub trait Codec<S, I, O>: Iterator<Item = Result<I>> {
    /// Constructor.
    fn new(inner: S, read_buf_size: usize) -> Self;

    /// Send the given value out using this codec.
    fn send(&mut self, value: O) -> Result<()>;
}

/// Blocking adapter to convert an underlying I/O stream, which implements
/// [`std::io::Read`] and [`std::io::Write`], into an [`Iterator`] producing
/// entities of type `I` and allowing for sending of entities of type `O`.
///
/// The blocking iterator terminates once the underlying reader terminates.
pub struct CodecImpl<E, D, O, I, S> {
    inner: S,
    _encoder: PhantomData<E>,
    _outgoing: PhantomData<O>,
    decoder: D,
    _incoming: PhantomData<I>,
    growable_read_buf: BytesMut,
    growable_write_buf: BytesMut,
    read_buf: Vec<u8>,
}

impl<E, D, O, I, S> Iterator for CodecImpl<E, D, O, I, S>
where
    D: Decoder<I>,
    S: Read,
{
    type Item = Result<I>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to decode a request from our internal read buffer first
            match self.decoder.decode(&mut self.growable_read_buf) {
                Ok(req_opt) => {
                    if let Some(req) = req_opt {
                        return Some(Ok(req));
                    }
                }
                Err(e) => return Some(Err(e)),
            }

            // If we don't have enough data to decode a message, try to read
            // more
            let bytes_read = match self.inner.read(self.read_buf.as_mut()) {
                Ok(br) => br,
                Err(e) => return Some(Err(e.into())),
            };
            if bytes_read == 0 {
                // The stream terminated
                return None;
            }
            self.growable_read_buf
                .extend_from_slice(&self.read_buf[..bytes_read]);
        }
    }
}

impl<E, D, O, I, S> Codec<S, I, O> for CodecImpl<E, D, O, I, S>
where
    E: Encoder<O>,
    D: Decoder<I> + Default,
    S: Read + Write,
{
    fn new(inner: S, read_buf_size: usize) -> Self {
        Self {
            inner,
            _encoder: Default::default(),
            _outgoing: Default::default(),
            decoder: Default::default(),
            _incoming: Default::default(),
            growable_read_buf: BytesMut::new(),
            growable_write_buf: BytesMut::new(),
            read_buf: vec![0_u8; read_buf_size],
        }
    }

    fn send(&mut self, value: O) -> Result<()> {
        E::encode(value, &mut self.growable_write_buf)?;
        loop {
            let bytes_written = self.inner.write(self.growable_write_buf.as_ref())?;
            if bytes_written == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "failed to write to underlying writer",
                )
                .into());
            }
            self.growable_write_buf.advance(bytes_written);
            if self.growable_write_buf.is_empty() {
                return Ok(self.inner.flush()?);
            }
        }
    }
}
