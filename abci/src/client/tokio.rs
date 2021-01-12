//! Tokio-based ABCI client.

use crate::client::Client;
use crate::codec::{TspDecoder, TspEncoder};
use crate::{Error, Result};
use async_trait::async_trait;
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use std::convert::TryInto;
use tendermint::abci::request::{Request, RequestInner};
use tendermint::abci::response::Response;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::codec::{Decoder, Encoder, Framed};

/// Tokio-based ABCI client for interacting with an ABCI server via a TCP
/// socket.
///
/// Not thread-safe, because it wraps a single outgoing TCP connection and the
/// underlying protocol doesn't support multiplexing. To submit requests in
/// parallel, create multiple TCP connections.
pub struct TokioClient {
    stream: Framed<TcpStream, ClientCodec>,
}

#[async_trait]
impl Client for TokioClient {
    async fn perform<R: RequestInner>(&mut self, req: R) -> Result<R::Response> {
        self.stream.send(req.into()).await?;
        let res: std::result::Result<R::Response, tendermint::Error> = self
            .stream
            .next()
            .await
            .ok_or(Error::ServerStreamTerminated)??
            .try_into();
        Ok(res?)
    }
}

impl TokioClient {
    /// Connect to the given ABCI server address.
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self {
            stream: Framed::new(stream, ClientCodec::default()),
        })
    }
}

/// Codec for the ABCI client.
///
/// Implements [`Encode`] for [`Request`]s and [`Decode`] for [`Response`]s.
pub struct ClientCodec {
    decoder: TspDecoder,
}

impl Default for ClientCodec {
    fn default() -> Self {
        Self {
            decoder: TspDecoder::new(),
        }
    }
}

impl Encoder<Request> for ClientCodec {
    type Error = Error;

    fn encode(&mut self, item: Request, mut dst: &mut BytesMut) -> Result<()> {
        TspEncoder::encode_request(item, &mut dst)
    }
}

impl Decoder for ClientCodec {
    type Item = Response;
    type Error = Error;

    fn decode(&mut self, mut src: &mut BytesMut) -> Result<Option<Self::Item>> {
        self.decoder.decode_response(&mut src)
    }
}
