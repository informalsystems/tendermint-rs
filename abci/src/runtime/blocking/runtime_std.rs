//! Rust standard library-based runtime.

use crate::codec::blocking::{ClientCodec, ServerCodec};
use crate::runtime::blocking::{Runtime, TcpListener, TcpStream};
use crate::Result;

/// Rust standard library-based runtime.
pub struct Std;

impl Runtime for Std {
    type TcpStream = StdTcpStream;
    type TcpListener = StdTcpListener;

    type ServerCodec = ServerCodec<std::net::TcpStream>;
    type ClientCodec = ClientCodec<std::net::TcpStream>;

    fn spawn_and_forget<T>(task: T)
    where
        T: FnOnce() + Send + 'static,
        T::Output: Send,
    {
        let _ = std::thread::spawn(move || {
            task();
        });
    }
}

/// Rust standard library TCP stream ([`std::net::TcpStream`]).
pub struct StdTcpStream(std::net::TcpStream);

impl TcpStream for StdTcpStream {
    type Inner = std::net::TcpStream;

    fn connect(addr: &str) -> Result<Self> {
        Ok(Self(std::net::TcpStream::connect(addr)?))
    }

    fn into_inner(self) -> Self::Inner {
        self.0
    }
}

/// Rust standard library TCP listener ([`std::net::TcpListener`]).
pub struct StdTcpListener(std::net::TcpListener);

impl TcpListener<StdTcpStream> for StdTcpListener {
    fn bind(addr: &str) -> Result<Self> {
        Ok(Self(std::net::TcpListener::bind(addr)?))
    }

    fn local_addr(&self) -> Result<String> {
        Ok(self.0.local_addr()?.to_string())
    }

    fn accept(&self) -> Result<(StdTcpStream, String)> {
        let (stream, addr) = self.0.accept()?;
        Ok((StdTcpStream(stream), addr.to_string()))
    }
}
