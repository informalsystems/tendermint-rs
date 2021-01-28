//! Non-blocking ABCI client.

use crate::client::DEFAULT_CLIENT_READ_BUF_SIZE;
use crate::codec::non_blocking::Codec;
use crate::runtime::non_blocking::{Runtime, TcpStream};
use crate::{Error, Result};
use futures::{SinkExt, StreamExt};
use std::convert::TryInto;
use tendermint::abci::request::RequestInner;
use tendermint::abci::{request, response};

/// A runtime-dependent non-blocking ABCI client.
pub struct Client<Rt: Runtime> {
    codec: Rt::ClientCodec,
}

impl<Rt: Runtime> Client<Rt> {
    /// Request that the ABCI server echo back the message in the given
    /// request.
    pub async fn echo(&mut self, req: request::Echo) -> Result<response::Echo> {
        self.perform(req).await
    }

    /// Provide information to the ABCI server about the Tendermint node in
    /// exchange for information about the application.
    pub async fn info(&mut self, req: request::Info) -> Result<response::Info> {
        self.perform(req).await
    }

    async fn perform<Req: RequestInner>(&mut self, req: Req) -> Result<Req::Response> {
        self.codec.send(req.into()).await?;
        let res: std::result::Result<Req::Response, tendermint::Error> = self
            .codec
            .next()
            .await
            .ok_or(Error::ServerStreamTerminated)??
            .try_into();
        Ok(res?)
    }
}

/// Builder for a non-blocking ABCI client.
pub struct ClientBuilder<Rt> {
    read_buf_size: usize,
    _runtime: std::marker::PhantomData<Rt>,
}

impl<Rt: Runtime> ClientBuilder<Rt> {
    /// Constructor allowing configuration of read buffer size.
    pub fn new(read_buf_size: usize) -> Self {
        Self {
            read_buf_size,
            _runtime: Default::default(),
        }
    }

    /// Connect to the ABCI server at the given network address.
    pub async fn connect<S>(self, addr: S) -> Result<Client<Rt>>
    where
        S: AsRef<str>,
    {
        let stream = Rt::TcpStream::connect(addr.as_ref()).await?;
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

#[cfg(feature = "runtime-tokio")]
/// Non-blocking ABCI client builder when using Tokio as your runtime.
pub type TokioClientBuilder = ClientBuilder<crate::runtime::non_blocking::runtime_tokio::Tokio>;

#[cfg(feature = "runtime-async-std")]
/// Non-blocking ABCI client builder when using `async-std` as your runtime.
pub type AsyncStdClientBuilder =
    ClientBuilder<crate::runtime::non_blocking::runtime_async_std::AsyncStd>;
