//! `async-std` runtime-specific types.

// TODO(thane): Implement something like tokio-util's `Framed` for async-std to
//              reduce code duplication in AsyncStdServerCodec and
//              AsyncStdServerCodec.

use crate::codec::{TspDecoder, TspEncoder};
use crate::runtime::{
    ChannelNotify, ClientCodec, Receiver, Runtime, Sender, ServerCodec, TaskSpawner, TcpListener,
    TcpStream,
};
use crate::{Error, Result};
use async_trait::async_trait;
use bytes::{Buf, BytesMut};
use futures::ready;
use futures::task::{Context, Poll};
use futures::{AsyncRead, AsyncWrite, Future, Sink, Stream};
use pin_project::pin_project;
use std::net::SocketAddr;
use std::pin::Pin;
use tendermint::abci::{request, response};

// The size of the read buffer we use when reading from a TCP stream. This is
// allocated each time a stream is polled for readiness to be read, so it's
// important that it's relatively small. Too small, however, and it'll increase
// CPU load due to increased decode/poll attempts.
//
// Tokio seems to get around this by using `unsafe` code in their buffer
// polling routine in `tokio-util`: https://github.com/tokio-rs/tokio/blob/198363f4f1a71cb98ffc0e9eaac335f669a5e1de/tokio-util/src/lib.rs#L106
// but we don't want to write our own `unsafe` code.
//
// As for a good number for general ABCI apps, that's something we should
// benchmark to determine.
// TODO(thane): Benchmark options here.
const CODEC_READ_BUF_SIZE: usize = 128;

/// `async-std` runtime.
pub struct AsyncStd;

impl Runtime for AsyncStd {
    type TcpStream = AsyncStdTcpStream;
    type TcpListener = AsyncStdTcpListener;
    type TaskSpawner = AsyncStdTaskSpawner;
    type ServerCodec = AsyncStdServerCodec;
    type ClientCodec = AsyncStdClientCodec;
    type ChannelNotify = AsyncStdChannelNotify;
}

pub struct AsyncStdTcpStream(async_std::net::TcpStream);

#[async_trait]
impl TcpStream for AsyncStdTcpStream {
    async fn connect(addr: &str) -> Result<Self> {
        Ok(Self(async_std::net::TcpStream::connect(addr).await?))
    }
}

pub struct AsyncStdTcpListener(async_std::net::TcpListener);

#[async_trait]
impl TcpListener<AsyncStdTcpStream> for AsyncStdTcpListener {
    async fn bind(addr: &str) -> Result<Self> {
        Ok(Self(async_std::net::TcpListener::bind(addr).await?))
    }

    fn local_addr(&self) -> Result<String> {
        Ok(self.0.local_addr()?.to_string())
    }

    async fn accept(&self) -> Result<(AsyncStdTcpStream, SocketAddr)> {
        let (stream, addr) = self.0.accept().await?;
        Ok((AsyncStdTcpStream(stream), addr))
    }
}

pub struct AsyncStdTaskSpawner;

impl TaskSpawner for AsyncStdTaskSpawner {
    fn spawn_and_forget<T>(task: T)
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let _ = async_std::task::spawn(task);
    }
}

#[pin_project]
pub struct AsyncStdServerCodec {
    #[pin]
    stream: async_std::net::TcpStream,
    read_buf: BytesMut,
    write_buf: BytesMut,
    decoder: TspDecoder,
}

impl ServerCodec<AsyncStdTcpStream> for AsyncStdServerCodec {
    fn from_tcp_stream(stream: AsyncStdTcpStream) -> Self {
        Self {
            stream: stream.0,
            read_buf: BytesMut::new(),
            write_buf: BytesMut::new(),
            decoder: TspDecoder::new(),
        }
    }
}

impl Stream for AsyncStdServerCodec {
    type Item = Result<request::Request>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut stream: Pin<&mut async_std::net::TcpStream> = this.stream;
        let read_buf: &mut BytesMut = this.read_buf;
        let decoder: &mut TspDecoder = this.decoder;
        let mut tmp_read_buf = [0_u8; CODEC_READ_BUF_SIZE];

        loop {
            // Try to decode a request from our existing buffer
            match decoder.decode_request(read_buf) {
                Ok(res) => {
                    if let Some(req) = res {
                        return Poll::Ready(Some(Ok(req)));
                    }
                }
                Err(e) => return Poll::Ready(Some(Err(e))),
            }

            // If we couldn't decode another request from the buffer, try to
            // fill up the buffer as much as we can
            let bytes_read = match ready!(stream.as_mut().poll_read(cx, &mut tmp_read_buf)) {
                Ok(br) => br,
                Err(e) => return Poll::Ready(Some(Err(e.into()))),
            };
            if bytes_read == 0 {
                // The stream terminated
                return Poll::Ready(None);
            }
            read_buf.extend_from_slice(&tmp_read_buf[..bytes_read]);
        }
    }
}

impl Sink<response::Response> for AsyncStdServerCodec {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: response::Response) -> Result<()> {
        let write_buf: &mut BytesMut = self.project().write_buf;
        TspEncoder::encode_response(item, write_buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let this = self.project();
        let mut stream: Pin<&mut async_std::net::TcpStream> = this.stream;
        let write_buf: &mut BytesMut = this.write_buf;

        while !write_buf.is_empty() {
            let bytes_written = match ready!(stream.as_mut().poll_write(cx, write_buf.as_ref())) {
                Ok(bw) => bw,
                Err(e) => return Poll::Ready(Err(e.into())),
            };
            if bytes_written == 0 {
                return Poll::Ready(Err(async_std::io::Error::new(
                    async_std::io::ErrorKind::WriteZero,
                    "failed to write to transport",
                )
                .into()));
            }
            write_buf.advance(bytes_written);
        }
        // Try to flush the underlying stream
        ready!(stream.poll_flush(cx))?;
        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        ready!(self.as_mut().poll_flush(cx))?;
        let stream: Pin<&mut async_std::net::TcpStream> = self.project().stream;
        ready!(stream.poll_close(cx))?;

        Poll::Ready(Ok(()))
    }
}

#[pin_project]
pub struct AsyncStdClientCodec {
    #[pin]
    stream: async_std::net::TcpStream,
    read_buf: BytesMut,
    write_buf: BytesMut,
    decoder: TspDecoder,
}

impl ClientCodec<AsyncStdTcpStream> for AsyncStdClientCodec {
    fn from_tcp_stream(stream: AsyncStdTcpStream) -> Self {
        Self {
            stream: stream.0,
            read_buf: BytesMut::new(),
            write_buf: BytesMut::new(),
            decoder: TspDecoder::new(),
        }
    }
}

impl Stream for AsyncStdClientCodec {
    type Item = Result<response::Response>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut stream: Pin<&mut async_std::net::TcpStream> = this.stream;
        let read_buf: &mut BytesMut = this.read_buf;
        let decoder: &mut TspDecoder = this.decoder;
        let mut tmp_read_buf = [0_u8; CODEC_READ_BUF_SIZE];

        loop {
            // Try to decode a response from our existing buffer
            match decoder.decode_response(read_buf) {
                Ok(res_opt) => {
                    if let Some(res) = res_opt {
                        return Poll::Ready(Some(Ok(res)));
                    }
                }
                Err(e) => return Poll::Ready(Some(Err(e))),
            }

            // If we couldn't decode another request from the buffer, try to
            // fill up the buffer as much as we can
            let bytes_read = match ready!(stream.as_mut().poll_read(cx, &mut tmp_read_buf)) {
                Ok(br) => br,
                Err(e) => return Poll::Ready(Some(Err(e.into()))),
            };
            if bytes_read == 0 {
                // The stream terminated
                return Poll::Ready(None);
            }
            read_buf.extend_from_slice(&tmp_read_buf[..bytes_read]);
        }
    }
}

impl Sink<request::Request> for AsyncStdClientCodec {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: request::Request) -> Result<()> {
        let write_buf: &mut BytesMut = self.project().write_buf;
        TspEncoder::encode_request(item, write_buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let this = self.project();
        let mut stream: Pin<&mut async_std::net::TcpStream> = this.stream;
        let write_buf: &mut BytesMut = this.write_buf;

        while !write_buf.is_empty() {
            let bytes_written = match ready!(stream.as_mut().poll_write(cx, write_buf.as_ref())) {
                Ok(bw) => bw,
                Err(e) => return Poll::Ready(Err(e.into())),
            };
            if bytes_written == 0 {
                return Poll::Ready(Err(async_std::io::Error::new(
                    async_std::io::ErrorKind::WriteZero,
                    "failed to write to transport",
                )
                .into()));
            }
            write_buf.advance(bytes_written);
        }
        // Try to flush the underlying stream
        ready!(stream.poll_flush(cx))?;
        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        ready!(self.as_mut().poll_flush(cx))?;
        let stream: Pin<&mut async_std::net::TcpStream> = self.project().stream;
        ready!(stream.poll_close(cx))?;

        Poll::Ready(Ok(()))
    }
}

pub struct AsyncStdChannelNotify;

impl ChannelNotify for AsyncStdChannelNotify {
    type Sender = AsyncStdSender<()>;
    type Receiver = AsyncStdReceiver<()>;

    fn unbounded() -> (Self::Sender, Self::Receiver) {
        let (tx, rx) = async_channel::unbounded();
        (AsyncStdSender(tx), AsyncStdReceiver(rx))
    }
}

pub struct AsyncStdSender<T>(async_channel::Sender<T>);

#[async_trait]
impl<T: Send> Sender<T> for AsyncStdSender<T> {
    async fn send(&self, value: T) -> Result<()> {
        self.0
            .send(value)
            .await
            .map_err(|e| Error::ChannelSend(e.to_string()))
    }
}

pub struct AsyncStdReceiver<T>(async_channel::Receiver<T>);

#[async_trait]
impl<T: Send> Receiver<T> for AsyncStdReceiver<T> {
    async fn recv(&mut self) -> Result<T> {
        self.0
            .recv()
            .await
            .map_err(|e| Error::ChannelRecv(e.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use async_std::task::JoinHandle;
    use futures::{SinkExt, StreamExt};
    use std::convert::TryInto;

    #[async_std::test]
    async fn codec() {
        let requests = (0..5)
            .map(|r| request::Echo {
                message: format!("echo {}", r),
            })
            .collect::<Vec<request::Echo>>();
        let listener = async_std::net::TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap();
        let local_addr = listener.local_addr().unwrap().to_string();
        let client_requests = requests.clone();
        let client_handle: JoinHandle<Vec<response::Echo>> = async_std::task::spawn(async move {
            let client_stream = async_std::net::TcpStream::connect(local_addr)
                .await
                .unwrap();
            let mut codec = AsyncStdClientCodec::from_tcp_stream(AsyncStdTcpStream(client_stream));
            let mut received_responses = Vec::new();

            for req in client_requests {
                codec.send(req.into()).await.unwrap();
                let res: response::Echo = codec.next().await.unwrap().unwrap().try_into().unwrap();
                received_responses.push(res);
            }

            received_responses
        });

        let (server_stream, _) = listener.accept().await.unwrap();
        let mut codec = AsyncStdServerCodec::from_tcp_stream(AsyncStdTcpStream(server_stream));

        let mut received_requests = Vec::new();
        while let Some(result) = codec.next().await {
            let request: request::Echo = result.unwrap().try_into().unwrap();
            codec
                .send(
                    response::Echo {
                        message: request.message.clone(),
                    }
                    .into(),
                )
                .await
                .unwrap();
            received_requests.push(request);
        }
        let received_responses = client_handle.await;
        assert_eq!(received_requests.len(), requests.len());
        assert_eq!(received_requests, requests);
        assert_eq!(received_responses.len(), requests.len());
    }
}
