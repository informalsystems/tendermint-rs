//! Tokio-based non-blocking runtime.

use crate::codec::non_blocking::{ClientCodec, ServerCodec};
use crate::runtime::non_blocking::{
    ChannelNotify, Receiver, Runtime, Sender, TcpListener, TcpStream,
};
use crate::{Error, Result};
use async_trait::async_trait;
use futures::task::{Context, Poll};
use futures::{ready, Future};
use pin_project::pin_project;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct Tokio;

impl Runtime for Tokio {
    type TcpStream = TokioTcpStream;
    type TcpListener = TokioTcpListener;

    type ServerCodec = ServerCodec<FuturesTcpStream>;
    type ClientCodec = ClientCodec<FuturesTcpStream>;
    type ChannelNotify = TokioChannelNotify;

    fn spawn_and_forget<T>(task: T)
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let _ = tokio::spawn(task);
    }
}

/// A wrapper for [`tokio::net::TcpStream`] that implements
/// [`futures::AsyncRead`] and [`futures::AsyncWrite`] to ensure compatibility
/// with our non-blocking (futures-based) interfaces.
#[pin_project]
pub struct FuturesTcpStream(#[pin] tokio::net::TcpStream);

impl futures::AsyncRead for FuturesTcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let mut buf = ReadBuf::new(buf);
        ready!(self.project().0.poll_read(cx, &mut buf)?);
        Poll::Ready(Ok(buf.filled().len()))
    }
}

impl futures::AsyncWrite for FuturesTcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        self.project().0.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.project().0.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.project().0.poll_shutdown(cx)
    }
}

pub struct TokioTcpStream(FuturesTcpStream);

#[async_trait]
impl TcpStream for TokioTcpStream {
    type Inner = FuturesTcpStream;

    async fn connect(addr: &str) -> Result<Self> {
        Ok(Self(FuturesTcpStream(
            tokio::net::TcpStream::connect(addr).await?,
        )))
    }

    fn into_inner(self) -> Self::Inner {
        self.0
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

    async fn accept(&self) -> Result<(TokioTcpStream, String)> {
        let (stream, addr) = self.0.accept().await?;
        Ok((TokioTcpStream(FuturesTcpStream(stream)), addr.to_string()))
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
