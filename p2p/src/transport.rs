use std::io::{Read, Write};
use std::net::SocketAddr;
use std::time::Duration;

use tendermint::public_key::PublicKey;

use ed25519_dalek as ed25519;
use eyre::Result;

pub mod mconnection;

pub struct BindInfo {
    pub addr: SocketAddr,
    pub private_key: ed25519::Keypair,
    pub listen_addrs: Vec<SocketAddr>,
}

pub struct ConnectInfo {
    pub addr: SocketAddr,
    pub timeout: Duration,
}

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
