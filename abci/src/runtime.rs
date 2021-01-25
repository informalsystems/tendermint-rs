//! Abstractions for facilitating runtime-independent code.

#[cfg(all(feature = "async", feature = "runtime-async-std"))]
pub mod async_std;
#[cfg(all(not(feature = "async"), feature = "runtime-std"))]
pub mod std;
#[cfg(all(feature = "async", feature = "runtime-tokio"))]
pub mod tokio;

// The runtime interface included here depends on whether the `async` feature
// flag is enabled or not.
pub use crate::runtime::interface::{
    ChannelNotify, ClientCodec, Receiver, Sender, ServerCodec, TaskSpawner, TcpListener, TcpStream,
};

/// Implemented by each runtime we support.
pub trait Runtime: 'static {
    type TcpStream: TcpStream;
    type TcpListener: TcpListener<Self::TcpStream>;
    type TaskSpawner: TaskSpawner;
    type ServerCodec: ServerCodec<Self::TcpStream>;
    type ClientCodec: ClientCodec<Self::TcpStream>;

    // TODO(thane): Make this generic once GATs are stable (see
    //              https://rust-lang.github.io/rfcs/1598-generic_associated_types.html)
    type ChannelNotify: ChannelNotify;
}

#[cfg(feature = "async")]
mod interface {
    use crate::{Error, Result};
    use async_trait::async_trait;
    use futures::{Future, Sink, Stream};
    use std::net::SocketAddr;
    use tendermint::abci::{request, response};

    #[async_trait]
    pub trait TcpStream: Sized + Send {
        async fn connect(addr: &str) -> Result<Self>;
    }

    #[async_trait]
    pub trait TcpListener<T: TcpStream>: Sized {
        /// Bind this listener to the given address.
        async fn bind(addr: &str) -> Result<Self>;

        /// Returns the string representation of this listener's local address.
        fn local_addr(&self) -> Result<String>;

        /// Attempt to accept an incoming connection.
        async fn accept(&self) -> Result<(T, SocketAddr)>;
    }

    pub trait TaskSpawner {
        /// Spawn an asynchronous task without caring about its result.
        fn spawn_and_forget<T>(task: T)
        where
            T: Future + Send + 'static,
            T::Output: Send + 'static;
    }

    pub trait ServerCodec<S: TcpStream>:
        Stream<Item = Result<request::Request>>
        + Sink<response::Response, Error = Error>
        + Unpin
        + Send
    {
        fn from_tcp_stream(stream: S) -> Self;
    }

    pub trait ClientCodec<S: TcpStream>:
        Sink<request::Request, Error = Error>
        + Stream<Item = Result<response::Response>>
        + Unpin
        + Send
    {
        fn from_tcp_stream(stream: S) -> Self;
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
}

#[cfg(not(feature = "async"))]
mod interface {
    use crate::Result;
    use std::net::SocketAddr;
    use tendermint::abci::{request, response};

    pub trait TcpStream: Sized + Send {
        fn connect(addr: &str) -> Result<Self>;
    }

    pub trait TcpListener<T: TcpStream>: Sized {
        /// Bind this listener to the given address.
        fn bind(addr: &str) -> Result<Self>;

        /// Returns the string representation of this listener's local address.
        fn local_addr(&self) -> Result<String>;

        /// Attempt to accept an incoming connection.
        fn accept(&self) -> Result<(T, SocketAddr)>;
    }

    pub trait TaskSpawner {
        /// Spawn a task in a separate thread without caring about its result.
        fn spawn_and_forget<T>(task: T)
        where
            T: FnOnce() + Send + 'static,
            T::Output: Send;
    }

    pub trait ServerCodec<S: TcpStream>: Iterator<Item = Result<request::Request>> {
        fn from_tcp_stream(stream: S) -> Self;

        fn send(&mut self, res: response::Response) -> Result<()>;
    }

    pub trait ClientCodec<S: TcpStream>: Iterator<Item = Result<response::Response>> {
        fn from_tcp_stream(stream: S) -> Self;

        fn send(&mut self, req: request::Request) -> Result<()>;
    }

    /// The sending end of a channel.
    pub trait Sender<T> {
        fn send(&self, value: T) -> Result<()>;
    }

    /// The receiving end of a channel.
    pub trait Receiver<T> {
        fn recv(&self) -> Result<T>;
    }

    /// A simple notification channel.
    pub trait ChannelNotify {
        type Sender: Sender<()>;
        type Receiver: Receiver<()>;

        /// Construct an unbounded channel.
        fn unbounded() -> (Self::Sender, Self::Receiver);
    }
}
