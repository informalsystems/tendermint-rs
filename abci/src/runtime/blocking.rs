//! Blocking API for blocking runtimes.

#[cfg(feature = "runtime-std")]
pub mod runtime_std;

use crate::codec::blocking::Codec;
use crate::Result;
use std::io::{Read, Write};
use tendermint::abci::{request, response};

/// Implemented by each blocking runtime we support.
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

    /// Spawn a task in a separate thread without caring about its result.
    fn spawn_and_forget<T>(task: T)
    where
        T: FnOnce() + Send + 'static,
        T::Output: Send;
}

pub trait TcpStream: Sized + Send {
    type Inner: Read + Write;

    fn connect(addr: &str) -> Result<Self>;

    fn into_inner(self) -> Self::Inner;
}

pub trait TcpListener<T: TcpStream>: Sized {
    /// Bind this listener to the given address.
    fn bind(addr: &str) -> Result<Self>;

    /// Returns the string representation of this listener's local address.
    fn local_addr(&self) -> Result<String>;

    /// Attempt to accept an incoming connection.
    ///
    /// On success, returns a TCP stream and a string representation of its address.
    fn accept(&self) -> Result<(T, String)>;
}

/// The sending end of a channel.
pub trait Sender<T> {
    fn send(&self, value: T) -> Result<()>;
}

/// The receiving end of a channel.
pub trait Receiver<T> {
    fn recv(&self) -> Result<T>;
}
