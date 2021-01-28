//! Blocking ABCI client.

use crate::client::DEFAULT_CLIENT_READ_BUF_SIZE;
use crate::codec::blocking::Codec;
use crate::runtime::blocking::{Runtime, TcpStream};
use crate::{Error, Result};
use std::convert::TryInto;
use tendermint::abci::request::RequestInner;
use tendermint::abci::{request, response};

/// A runtime-dependent blocking ABCI client.
pub struct Client<Rt: Runtime> {
    codec: Rt::ClientCodec,
}

impl<Rt: Runtime> Client<Rt> {
    /// Request that the ABCI server echo back the message in the given
    /// request.
    pub fn echo(&mut self, req: request::Echo) -> Result<response::Echo> {
        self.perform(req)
    }

    /// Provide information to the ABCI server about the Tendermint node in
    /// exchange for information about the application.
    pub fn info(&mut self, req: request::Info) -> Result<response::Info> {
        self.perform(req)
    }

    fn perform<Req: RequestInner>(&mut self, req: Req) -> Result<Req::Response> {
        self.codec.send(req.into())?;
        match self.codec.next() {
            Some(result) => {
                let res = result?;
                Ok(res.try_into()?)
            }
            None => Err(Error::ServerStreamTerminated),
        }
    }
}

/// Builder for a blocking ABCI client.
pub struct ClientBuilder<Rt> {
    read_buf_size: usize,
    _runtime: std::marker::PhantomData<Rt>,
}

impl<Rt: Runtime> ClientBuilder<Rt> {
    /// Constructor allowing customization of the client's read buffer size.
    pub fn new(read_buf_size: usize) -> Self {
        Self {
            read_buf_size,
            _runtime: Default::default(),
        }
    }

    /// Constructor for our [`Client`] instance, which attempts to connect to
    /// the given address.
    pub fn connect<S>(self, addr: S) -> Result<Client<Rt>>
    where
        S: AsRef<str>,
    {
        let stream = Rt::TcpStream::connect(addr.as_ref())?;
        Ok(Client {
            codec: Rt::ClientCodec::new(stream.into_inner(), self.read_buf_size),
        })
    }
}

impl<Rt: Runtime> Default for ClientBuilder<Rt> {
    fn default() -> Self {
        Self {
            read_buf_size: DEFAULT_CLIENT_READ_BUF_SIZE,
            _runtime: Default::default(),
        }
    }
}

#[cfg(feature = "runtime-std")]
/// Blocking ABCI client builder when using Rust's standard library as your
/// runtime.
pub type StdClientBuilder = ClientBuilder<crate::runtime::blocking::runtime_std::Std>;
