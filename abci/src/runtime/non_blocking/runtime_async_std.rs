//! `async-std`-based non-blocking runtime.

use crate::codec::non_blocking::{ClientCodec, ServerCodec};
use crate::runtime::non_blocking::{
    ChannelNotify, Receiver, Runtime, Sender, TcpListener, TcpStream,
};
use crate::{Error, Result};
use async_trait::async_trait;
use futures::Future;

pub struct AsyncStd;

impl Runtime for AsyncStd {
    type TcpStream = AsyncStdTcpStream;
    type TcpListener = AsyncStdTcpListener;

    type ServerCodec = ServerCodec<async_std::net::TcpStream>;
    type ClientCodec = ClientCodec<async_std::net::TcpStream>;
    type ChannelNotify = AsyncStdChannelNotify;

    fn spawn_and_forget<T>(task: T)
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let _ = async_std::task::spawn(task);
    }
}

pub struct AsyncStdTcpStream(async_std::net::TcpStream);

#[async_trait]
impl TcpStream for AsyncStdTcpStream {
    type Inner = async_std::net::TcpStream;

    async fn connect(addr: &str) -> Result<Self> {
        Ok(Self(async_std::net::TcpStream::connect(addr).await?))
    }

    fn into_inner(self) -> Self::Inner {
        self.0
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

    async fn accept(&self) -> Result<(AsyncStdTcpStream, String)> {
        let (stream, addr) = self.0.accept().await?;
        Ok((AsyncStdTcpStream(stream), addr.to_string()))
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

pub struct AsyncStdChannelNotify;

impl ChannelNotify for AsyncStdChannelNotify {
    type Sender = AsyncStdSender<()>;
    type Receiver = AsyncStdReceiver<()>;

    fn unbounded() -> (Self::Sender, Self::Receiver) {
        let (tx, rx) = async_channel::unbounded();
        (AsyncStdSender(tx), AsyncStdReceiver(rx))
    }
}
