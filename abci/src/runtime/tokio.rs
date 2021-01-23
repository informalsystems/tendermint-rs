//! Tokio runtime-specific types.

use crate::codec::{TspDecoder, TspEncoder};
use crate::runtime::{
    ChannelNotify, ClientCodec, Receiver, Runtime, Sender, ServerCodec, TaskSpawner, TcpListener,
    TcpStream,
};
use crate::{Error, Result};
use async_trait::async_trait;
use bytes::BytesMut;
use futures::task::{Context, Poll};
use futures::{Future, Sink, Stream};
use pin_project::pin_project;
use std::net::SocketAddr;
use std::pin::Pin;
use tendermint::abci::{request, response};
use tokio_util::codec::{Decoder, Encoder, Framed};

/// Tokio runtime.
pub struct Tokio;

impl Runtime for Tokio {
    type TcpStream = TokioTcpStream;
    type TcpListener = TokioTcpListener;
    type TaskSpawner = TokioTaskSpawner;
    type ServerCodec = TokioServerCodec;
    type ClientCodec = TokioClientCodec;
    type ChannelNotify = TokioChannelNotify;
}

pub struct TokioTcpStream(tokio::net::TcpStream);

#[async_trait]
impl TcpStream for TokioTcpStream {
    async fn connect(addr: &str) -> Result<Self> {
        Ok(Self(tokio::net::TcpStream::connect(addr).await?))
    }
}

pub struct TokioTcpListener(tokio::net::TcpListener);

#[async_trait]
impl TcpListener<TokioTcpStream> for TokioTcpListener {
    async fn bind(addr: &str) -> Result<Self> {
        Ok(Self(tokio::net::TcpListener::bind(addr).await?))
    }

    fn local_addr(&self) -> Result<String> {
        Ok(self.0.local_addr()?.to_string())
    }

    async fn accept(&self) -> Result<(TokioTcpStream, SocketAddr)> {
        let (stream, addr) = self.0.accept().await?;
        Ok((TokioTcpStream(stream), addr))
    }
}

pub struct TokioTaskSpawner;

impl TaskSpawner for TokioTaskSpawner {
    fn spawn_and_forget<T>(task: T)
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let _ = tokio::spawn(task);
    }
}

#[pin_project]
pub struct TokioServerCodec(#[pin] Framed<tokio::net::TcpStream, TspServerCodec>);

impl ServerCodec<TokioTcpStream> for TokioServerCodec {
    fn from_tcp_stream(stream: TokioTcpStream) -> Self {
        Self(Framed::new(stream.0, TspServerCodec::default()))
    }
}

impl Stream for TokioServerCodec {
    type Item = Result<request::Request>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().0.poll_next(cx)
    }
}

impl Sink<response::Response> for TokioServerCodec {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().0.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: response::Response) -> Result<()> {
        self.project().0.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().0.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().0.poll_close(cx)
    }
}

pub struct TspServerCodec {
    decoder: TspDecoder,
}

impl Default for TspServerCodec {
    fn default() -> Self {
        Self {
            decoder: TspDecoder::new(),
        }
    }
}

impl Encoder<response::Response> for TspServerCodec {
    type Error = Error;

    fn encode(&mut self, item: response::Response, dst: &mut BytesMut) -> Result<()> {
        TspEncoder::encode_response(item, dst)
    }
}

impl Decoder for TspServerCodec {
    type Item = request::Request;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        self.decoder.decode_request(src)
    }
}

#[pin_project]
pub struct TokioClientCodec(#[pin] Framed<tokio::net::TcpStream, TspClientCodec>);

impl ClientCodec<TokioTcpStream> for TokioClientCodec {
    fn from_tcp_stream(stream: TokioTcpStream) -> Self {
        Self(Framed::new(stream.0, TspClientCodec::default()))
    }
}

impl Stream for TokioClientCodec {
    type Item = Result<response::Response>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().0.poll_next(cx)
    }
}

impl Sink<request::Request> for TokioClientCodec {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().0.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: request::Request) -> Result<()> {
        self.project().0.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().0.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.project().0.poll_close(cx)
    }
}

pub struct TspClientCodec {
    decoder: TspDecoder,
}

impl Default for TspClientCodec {
    fn default() -> Self {
        Self {
            decoder: TspDecoder::new(),
        }
    }
}

impl Encoder<request::Request> for TspClientCodec {
    type Error = Error;

    fn encode(&mut self, item: request::Request, dst: &mut BytesMut) -> Result<()> {
        TspEncoder::encode_request(item, dst)
    }
}

impl Decoder for TspClientCodec {
    type Item = response::Response;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        self.decoder.decode_response(src)
    }
}

pub struct TokioChannelNotify;

impl ChannelNotify for TokioChannelNotify {
    type Sender = TokioSender<()>;
    type Receiver = TokioReceiver<()>;

    fn unbounded() -> (Self::Sender, Self::Receiver) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        (TokioSender(tx), TokioReceiver(rx))
    }
}

pub struct TokioSender<T>(tokio::sync::mpsc::UnboundedSender<T>);

#[async_trait]
impl<T: Send> Sender<T> for TokioSender<T> {
    async fn send(&self, value: T) -> Result<()> {
        self.0
            .send(value)
            .map_err(|e| Error::ChannelSend(e.to_string()))
    }
}

pub struct TokioReceiver<T>(tokio::sync::mpsc::UnboundedReceiver<T>);

#[async_trait]
impl<T: Send> Receiver<T> for TokioReceiver<T> {
    async fn recv(&mut self) -> Result<T> {
        self.0.recv().await.ok_or(Error::ChannelSenderClosed)
    }
}
