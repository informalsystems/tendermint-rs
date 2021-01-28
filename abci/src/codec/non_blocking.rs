//! `async` (futures)-compatible API for frame en/decoding.

use crate::codec::{
    Decoder, Encoder, RequestDecoder, RequestEncoder, ResponseDecoder, ResponseEncoder,
};
use crate::{Error, Result};
use bytes::{Buf, BytesMut};
use futures::task::{Context, Poll};
use futures::{ready, AsyncRead, AsyncWrite, Sink, Stream};
use pin_project::pin_project;
use std::marker::PhantomData;
use std::pin::Pin;
use tendermint::abci::request::Request;
use tendermint::abci::response::Response;

/// An ABCI server decodes incoming [`tendermint::abci::request::Request`]s
/// and encodes outgoing [`tendermint::abci::response::Response`]s. The stream
/// `S` must implement [`futures::AsyncRead`] and [`futures::AsyncWrite`].
pub type ServerCodec<S> = CodecImpl<ResponseEncoder, RequestDecoder, Response, Request, S>;

/// An ABCI client encodes outgoing [`tendermint::abci::request::Request`]s
/// and decodes incoming [`tendermint::abci::response::Response`]s. The stream
/// `S` must implement [`futures::AsyncRead`] and [`futures::AsyncWrite`].
pub type ClientCodec<S> = CodecImpl<RequestEncoder, ResponseDecoder, Request, Response, S>;

/// A non-blocking codec that allows us to iterate over a [`futures::Stream`]
/// producing values of type `Result<I>`. It also implements [`futures::Sink`],
/// allowing us to send values of type `O`.
pub trait Codec<S, I, O>: Stream<Item = Result<I>> + Sink<O, Error = Error> + Send + Unpin {
    /// Constructor.
    fn new(inner: S, read_buf_size: usize) -> Self;
}

/// Non-blocking adapter to convert an underlying I/O stream, which implements
/// [`futures::AsyncRead`] and [`futures::AsyncWrite`], into a
/// [`futures::Stream`] producing entities of type `I` and a [`futures::Sink`]
/// allowing for sending of entities of type `O`.
#[pin_project]
pub struct CodecImpl<E, D, O, I, S> {
    #[pin]
    inner: S,
    _encoder: PhantomData<E>,
    _output: PhantomData<O>,
    decoder: D,
    _input: PhantomData<I>,
    growable_read_buf: BytesMut,
    growable_write_buf: BytesMut,
    read_buf: Vec<u8>,
}

impl<E, D, O, I, S> Stream for CodecImpl<E, D, O, I, S>
where
    D: Decoder<I>,
    S: AsyncRead,
{
    type Item = Result<I>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut inner: Pin<&mut S> = this.inner;
        let growable_read_buf: &mut BytesMut = this.growable_read_buf;
        let read_buf: &mut Vec<u8> = this.read_buf;
        let decoder: &mut D = this.decoder;

        loop {
            // Try to decode another input value from our existing read buffer
            // if we can without resorting to I/O.
            match decoder.decode(growable_read_buf) {
                Ok(res) => {
                    if let Some(val) = res {
                        return Poll::Ready(Some(Ok(val)));
                    }
                }
                Err(e) => return Poll::Ready(Some(Err(e))),
            }

            // If we don't have enough data to decode an input value, try to
            // read some more from the underlying reader.
            let bytes_read = match ready!(inner.as_mut().poll_read(cx, read_buf.as_mut())) {
                Ok(br) => br,
                Err(e) => return Poll::Ready(Some(Err(e.into()))),
            };
            if bytes_read == 0 {
                // The underlying stream terminated
                return Poll::Ready(None);
            }
            growable_read_buf.extend_from_slice(&read_buf[..bytes_read]);
        }
    }
}

impl<E, D, O, I, S> Sink<O> for CodecImpl<E, D, O, I, S>
where
    E: Encoder<O>,
    S: AsyncWrite,
{
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: O) -> Result<()> {
        let write_buf: &mut BytesMut = self.project().growable_write_buf;
        E::encode(item, write_buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let this = self.project();
        let mut inner: Pin<&mut S> = this.inner;
        let write_buf: &mut BytesMut = this.growable_write_buf;

        while !write_buf.is_empty() {
            let bytes_written = match ready!(inner.as_mut().poll_write(cx, write_buf.as_ref())) {
                Ok(bw) => bw,
                Err(e) => return Poll::Ready(Err(e.into())),
            };
            if bytes_written == 0 {
                return Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "failed to write to underlying stream",
                )
                .into()));
            }
            write_buf.advance(bytes_written);
        }
        // Try to flush the underlying stream
        ready!(inner.poll_flush(cx))?;
        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        ready!(self.as_mut().poll_flush(cx))?;
        let inner: Pin<&mut S> = self.project().inner;
        ready!(inner.poll_close(cx))?;

        Poll::Ready(Ok(()))
    }
}

impl<E, D, O, I, S> Codec<S, I, O> for CodecImpl<E, D, O, I, S>
where
    E: Encoder<O> + Send,
    D: Decoder<I> + Default + Send,
    O: Send,
    I: Send,
    S: AsyncRead + AsyncWrite + Send + Unpin,
{
    fn new(inner: S, read_buf_size: usize) -> Self {
        Self {
            inner,
            _encoder: Default::default(),
            _output: Default::default(),
            decoder: Default::default(),
            _input: Default::default(),
            growable_read_buf: BytesMut::new(),
            growable_write_buf: BytesMut::new(),
            read_buf: vec![0_u8; read_buf_size],
        }
    }
}
