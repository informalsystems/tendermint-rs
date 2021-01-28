//! Blocking ABCI client.

use crate::codec::ClientCodec;
use crate::{Error, Result};
use std::net::{TcpStream, ToSocketAddrs};
use tendermint_proto::abci::request;
use tendermint_proto::abci::response::Value;
use tendermint_proto::abci::{Request, RequestEcho, ResponseEcho};

/// The size of the read buffer for the client in its receiving of responses
/// from the server.
pub const DEFAULT_CLIENT_READ_BUF_SIZE: usize = 1024;

/// Builder for a blocking ABCI client.
pub struct ClientBuilder {
    read_buf_size: usize,
}

impl ClientBuilder {
    /// Builder constructor.
    pub fn new(read_buf_size: usize) -> Self {
        Self { read_buf_size }
    }

    /// Client constructor that attempts to connect to the given network
    /// address.
    pub fn connect<A: ToSocketAddrs>(self, addr: A) -> Result<Client> {
        let stream = TcpStream::connect(addr)?;
        Ok(Client {
            codec: ClientCodec::new(stream, self.read_buf_size),
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            read_buf_size: DEFAULT_CLIENT_READ_BUF_SIZE,
        }
    }
}

/// Blocking ABCI client.
pub struct Client {
    codec: ClientCodec<TcpStream>,
}

impl Client {
    /// Ask the ABCI server to echo back a message.
    pub fn echo(&mut self, request: RequestEcho) -> Result<ResponseEcho> {
        self.codec.send(Request {
            value: Some(request::Value::Echo(request)),
        })?;
        let response = self
            .codec
            .next()
            .ok_or(Error::ServerConnectionTerminated)??;
        match response.value {
            Some(value) => match value {
                Value::Echo(response) => Ok(response),
                _ => Err(Error::UnexpectedServerResponseType(
                    "Echo".to_string(),
                    value,
                )),
            },
            None => Err(Error::MalformedServerResponse),
        }
    }
}
