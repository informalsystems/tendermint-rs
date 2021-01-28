//! Non-blocking runtime interface.

#[cfg(feature = "runtime-async-std")]
pub mod runtime_async_std;
#[cfg(feature = "runtime-tokio")]
pub mod runtime_tokio;

use crate::codec::non_blocking::Codec;
use crate::Result;
use async_trait::async_trait;
use futures::{AsyncRead, AsyncWrite, Future};
use tendermint::abci::{request, response};

/// Implemented by each non-blocking runtime we support.
pub trait Runtime: 'static {
    type TcpStream: TcpStream;
    type TcpListener: TcpListener<Self::TcpStream>;

    // Crate-specific types
    type ServerCodec: Codec<
        <<Self as Runtime>::TcpStream as TcpStream>::Inner,
        request::Request,
        response::Response,
    >;
    type ClientCodec: Codec<
        <<Self as Runtime>::TcpStream as TcpStream>::Inner,
        response::Response,
        request::Request,
    >;
    type ChannelNotify: ChannelNotify;

    /// Spawn an asynchronous task without caring about its result.
    fn spawn_and_forget<T>(task: T)
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static;
}

#[async_trait]
pub trait TcpStream: Sized + Send {
    type Inner: AsyncRead + AsyncWrite;

    async fn connect(addr: &str) -> Result<Self>;

    fn into_inner(self) -> Self::Inner;
}

#[async_trait]
pub trait TcpListener<T: TcpStream>: Sized {
    /// Bind this listener to the given address.
    async fn bind(addr: &str) -> Result<Self>;

    /// Returns the string representation of this listener's local address.
    fn local_addr(&self) -> Result<String>;

    /// Attempt to accept an incoming connection.
    async fn accept(&self) -> Result<(T, String)>;
}

/// The sending end of a channel.
#[async_trait]
pub trait Sender<T> {
    async fn send(&self, value: T) -> Result<()>;
}

/// The receiving end of a channel.
#[async_trait]
pub trait Receiver<T> {
    async fn recv(&mut self) -> Result<T>;
}

/// A simple notification channel.
pub trait ChannelNotify {
    type Sender: Sender<()>;
    type Receiver: Receiver<()>;

    /// Construct an unbounded channel.
    fn unbounded() -> (Self::Sender, Self::Receiver);
}
