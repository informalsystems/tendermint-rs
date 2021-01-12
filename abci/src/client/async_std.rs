//! `async-std`-based ABCI client.

use crate::codec::{TspDecoder, TspEncoder};
use crate::{Client, Result};
use async_std::net::{TcpStream, ToSocketAddrs};
use async_trait::async_trait;
use bytes::BytesMut;
use futures::{AsyncReadExt, AsyncWriteExt};
use std::convert::TryInto;
use tendermint::abci::request::RequestInner;

const CLIENT_READ_BUF_SIZE: usize = 4096;

/// `async-std`-based ABCI client for interacting with an ABCI server via a TCP
/// socket.
///
/// Not thread-safe, because it wraps a single outgoing TCP connection and the
/// underlying protocol doesn't support multiplexing. To submit requests in
/// parallel, create multiple client instances.
pub struct AsyncStdClient {
    stream: TcpStream,
    read_buf: BytesMut,
    write_buf: BytesMut,
    decoder: TspDecoder,
}

#[async_trait]
impl Client for AsyncStdClient {
    async fn perform<R: RequestInner>(&mut self, req: R) -> Result<R::Response> {
        TspEncoder::encode_request(req.into(), &mut self.write_buf)?;
        self.stream.write(self.write_buf.as_ref()).await?;
        self.write_buf.clear();

        let mut read_buf = [0u8; CLIENT_READ_BUF_SIZE];
        loop {
            let bytes_read = self.stream.read(&mut read_buf).await?;
            self.read_buf.extend_from_slice(&read_buf[..bytes_read]);
            // Try to read a full response
            if let Some(response) = self.decoder.decode_response(&mut self.read_buf)? {
                return Ok(response.try_into()?);
            }
            // Otherwise continue reading into our read buffer
        }
    }
}

impl AsyncStdClient {
    /// Connect to the given ABCI server address.
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self {
            stream,
            read_buf: BytesMut::new(),
            write_buf: BytesMut::new(),
            decoder: TspDecoder::new(),
        })
    }
}
