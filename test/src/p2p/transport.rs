use std::net::SocketAddr;

use eyre::{Report, Result};

use tendermint::public_key::PublicKey;
use tendermint_p2p::transport::{
    BindInfo, ConnectInfo, Connection, Endpoint, StreamId, StreamSend, Transport,
};

struct MemoryStreamRead;

impl Iterator for MemoryStreamRead {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

struct MemoryStreamSend;

impl StreamSend for MemoryStreamSend {
    fn send(msg: Vec<u8>) -> Result<()> {
        todo!()
    }
}

pub struct MemoryConnection;

impl Connection for MemoryConnection {
    type Error = Report;
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
        stream_id: StreamId,
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

    fn connect(&self, info: ConnectInfo) -> Result<Self::Connection> {
        todo!()
    }

    fn listen_addrs(&self) -> Vec<SocketAddr> {
        todo!()
    }
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

    fn bind(self, bind_info: BindInfo) -> Result<(Self::Endpoint, Self::Incoming)> {
        todo!()
    }
    fn shutdown(&self) -> Result<()> {
        todo!()
    }
}
