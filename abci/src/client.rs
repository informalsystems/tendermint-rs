//! ABCI clients for interacting with ABCI servers.

use crate::runtime::{ClientCodec, Runtime, TcpStream};
use crate::{Error, Result};
use std::convert::TryInto;
use tendermint::abci::request::RequestInner;
use tendermint::abci::{request, response};

/// A runtime-dependent ABCI client.
pub struct Client<Rt: Runtime> {
    codec: Rt::ClientCodec,
}

#[cfg(feature = "async")]
impl<Rt: Runtime> Client<Rt> {
    /// Connect to the ABCI server at the given network address.
    pub async fn connect<S: AsRef<str>>(addr: S) -> Result<Self> {
        let stream = Rt::TcpStream::connect(addr.as_ref()).await?;
        Ok(Self {
            codec: Rt::ClientCodec::from_tcp_stream(stream),
        })
    }

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
        use futures::{SinkExt, StreamExt};

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
