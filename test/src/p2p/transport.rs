use std::net::SocketAddr;

use eyre::Result;

use tendermint::public_key::PublicKey;
use tendermint_p2p::transport::{
    BindInfo, ConnectInfo, Connection, Endpoint, StreamId, StreamSend, Transport,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub struct MemoryStreamRead;

impl Iterator for MemoryStreamRead {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct MemoryStreamSend;

impl StreamSend for MemoryStreamSend {
    fn send<B: AsRef<[u8]>>(_msg: B) -> Result<()> {
        todo!()
    }
}

pub struct MemoryConnection;

impl Connection for MemoryConnection {
    type Error = Error;
    type StreamRead = MemoryStreamRead;
    type StreamSend = MemoryStreamSend;

    fn advertised_addrs(&self) -> Vec<SocketAddr> {
        todo!()
    }
    fn close(&self) -> Result<()> {
        todo!()
    }
    fn local_addr(&self) -> SocketAddr {
        todo!()
    }
    fn open_bidirectional(
        &self,
        _stream_id: StreamId,
    ) -> Result<(Self::StreamRead, Self::StreamSend), Self::Error> {
        todo!()
    }
    fn public_key(&self) -> PublicKey {
        todo!()
    }
    fn remote_addr(&self) -> SocketAddr {
        todo!()
    }
}

pub struct MemoryEndpoint;

impl Endpoint for MemoryEndpoint {
    type Connection = MemoryConnection;

    fn connect(&self, _info: ConnectInfo) -> Result<Self::Connection> {
        todo!()
    }

    fn listen_addrs(&self) -> Vec<SocketAddr> {
        todo!()
    }
}

impl Drop for MemoryEndpoint {
    fn drop(&mut self) {}
}

pub struct MemoryIncoming;

impl Iterator for MemoryIncoming {
    type Item = Result<MemoryConnection>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct Memory {}

impl Transport for Memory {
    type Connection = MemoryConnection;
    type Endpoint = MemoryEndpoint;
    type Incoming = MemoryIncoming;

    fn bind(self, _bind_info: BindInfo) -> Result<(Self::Endpoint, Self::Incoming)> {
        todo!()
    }
}
