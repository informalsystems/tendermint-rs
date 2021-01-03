use std::fmt;
use std::io::{Read, Write};
use std::net::SocketAddr;

use tendermint::public_key::PublicKey;

use eyre::{eyre, Result};

use crate::peer::{self, Peer};

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

pub trait Connection: Drop + Send {
    type Error: 'static + fmt::Display + std::error::Error + Send + Sync;
    type Read: Read;
    type Write: Write;

    fn advertised_addrs(&self) -> Vec<SocketAddr>;
    fn close(&self) -> Result<()>;
    fn local_addr(&self) -> SocketAddr;
    fn open_bidirectional(
        &self,
        stream_id: &StreamId,
    ) -> Result<(Self::Read, Self::Write), Self::Error>;
    fn public_key(&self) -> PublicKey;
    fn remote_addr(&self) -> SocketAddr;
}

pub trait Endpoint: Drop + Send {
    type Connection;

    fn connect(&self, info: ConnectInfo) -> Result<Self::Connection>;
    fn listen_addrs(&self) -> SocketAddr;
}

pub trait Transport {
    type Connection: Connection;
    type Endpoint: Endpoint<Connection = <Self as Transport>::Connection>;
    type Incoming: Iterator<Item = Result<<Self as Transport>::Connection>> + Send;

    fn bind(&self, bind_info: BindInfo) -> Result<(Self::Endpoint, Self::Incoming)>;
    fn shutdown(&self) -> Result<()>;
}

pub trait State: private::Sealed {}
pub struct Stopped;
impl State for Stopped {}
pub struct Running<E, I> {
    endpoint: E,
    incoming: I,
}
impl<E, I> State for Running<E, I> {}

pub struct Protocol<T, St> {
    transport: T,

    state: St,
}

impl<T, St> Protocol<T, St> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(transport: T) -> Protocol<T, Stopped> {
        Protocol {
            transport,
            state: Stopped,
        }
    }
}

impl<T> Protocol<T, Stopped>
where
    T: Transport,
{
    pub fn start(
        self,
        bind_info: BindInfo,
    ) -> Result<Protocol<T, Running<T::Endpoint, T::Incoming>>> {
        let (endpoint, incoming) = self.transport.bind(bind_info)?;

        Ok(Protocol {
            transport: self.transport,
            state: Running { endpoint, incoming },
        })
    }
}

impl<T, E, I> Protocol<T, Running<E, I>>
where
    T: Transport,
    E: Endpoint,
    E::Connection: Connection,
    I: Iterator<Item = Result<E::Connection>>,
{
    pub fn accept(&mut self) -> Result<Peer<peer::Connected<E::Connection>>> {
        match self.state.incoming.next() {
            Some(res) => Ok(Peer::from(Direction::Incoming(res?))),
            None => Err(eyre!("accept stream terminated, listener likely gone")),
        }
    }

    pub fn connect(&self, info: ConnectInfo) -> Result<Peer<peer::Connected<E::Connection>>> {
        let connection = self.state.endpoint.connect(info)?;

        Ok(Peer::from(Direction::Outgoing(connection)))
    }

    pub fn stop(self) -> Result<Protocol<T, Stopped>> {
        self.transport.shutdown()?;

        Ok(Protocol {
            transport: self.transport,
            state: Stopped {},
        })
    }
}

mod private {
    use super::{Running, Stopped};
    /// Constraint for [sealed traits] under the `transport` module hierarchy.
    ///
    /// [sealed traits]: https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
    pub trait Sealed {}

    impl Sealed for Stopped {}
    impl<E, I> Sealed for Running<E, I> {}
}
