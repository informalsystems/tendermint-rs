use std::io::{Read, Write};
use std::net::SocketAddr;

use eyre::Result;

use tendermint::public_key::PublicKey;

pub struct BindInfo {
    pub addr: SocketAddr,
    pub advertise_addrs: Vec<SocketAddr>,
    pub public_key: PublicKey,
}

pub struct ConnectInfo {}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum StreamId {
    Pex,
}

pub enum Direction<Conn> {
    Incoming(Conn),
    Outgoing(Conn),
}

pub trait Connection: Send {
    type Error: std::error::Error + Send + Sync + 'static;
    type Read: Read;
    type Write: Write;

    fn advertised_addrs(&self) -> Vec<SocketAddr>;
    fn close(&self) -> Result<()>;
    fn local_addr(&self) -> SocketAddr;
    fn open_bidirectional(
        &self,
        stream_id: StreamId,
    ) -> Result<(Self::Read, Self::Write), Self::Error>;
    fn public_key(&self) -> PublicKey;
    fn remote_addr(&self) -> SocketAddr;
}

pub trait Endpoint: Send {
    type Connection;

    fn connect(&self, info: ConnectInfo) -> Result<Self::Connection>;
    fn listen_addrs(&self) -> Vec<SocketAddr>;
}

pub trait Transport {
    type Connection: Connection;
    type Endpoint: Endpoint<Connection = <Self as Transport>::Connection>;
    type Incoming: Iterator<Item = Result<<Self as Transport>::Connection>> + Send;

    fn bind(self, bind_info: BindInfo) -> Result<(Self::Endpoint, Self::Incoming)>;
    fn shutdown(&self) -> Result<()>;
}
