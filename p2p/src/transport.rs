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

pub trait StreamSend {
    fn send<B: AsRef<[u8]>>(msg: B) -> Result<()>;
}

pub trait Connection: Send {
    type Error: std::error::Error + Send + Sync + 'static;
    type StreamRead: Iterator<Item = Result<Vec<u8>>> + Send;
    type StreamSend: StreamSend;

    fn advertised_addrs(&self) -> Vec<SocketAddr>;
    fn close(&self) -> Result<()>;
    fn local_addr(&self) -> SocketAddr;
    fn open_bidirectional(
        &self,
        stream_id: StreamId,
    ) -> Result<(Self::StreamRead, Self::StreamSend), Self::Error>;
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
    type Endpoint: Endpoint<Connection = <Self as Transport>::Connection> + Drop;
    type Incoming: Iterator<Item = Result<<Self as Transport>::Connection>> + Send;

    /// Consumes the transport to bind the resources in exchange for the `Endpoint` and `Incoming`
    /// stream.
    ///
    /// # Errors
    ///
    /// * if resource allocation fails for lack of priviliges or being not available.
    fn bind(self, bind_info: BindInfo) -> Result<(Self::Endpoint, Self::Incoming)>;
}
