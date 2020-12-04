use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use crate::peer::{self, Peer};

// TODO(xla): Use actual PeerId type.
type PeerId = String;

pub struct BindInfo {
    pub addr: SocketAddr,
    pub advertise_addrs: Vec<SocketAddr>,
    pub peer_id: PeerId,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub trait Connection: Clone {
    type Stream;

    fn advertised_addrs(&self) -> Vec<SocketAddr>;
    fn close(&self) -> Result<(), Error>;
    fn open(&self) -> Result<Self::Stream, Error>;
    fn peer_id(&self) -> PeerId;
    fn remote_addr(&self) -> SocketAddr;
}

pub trait Endpoint {
    type Connection;

    fn connect(&self) -> Result<Self::Connection, Error>;
    fn listen_addrs(&self) -> SocketAddr;
}

pub trait Transport {
    type Connection: Connection;
    type Endpoint: Endpoint<Connection = <Self as Transport>::Connection>;
    type Incoming: Iterator<Item = Result<<Self as Transport>::Connection, Error>>;

    fn bind(&self, bind_info: BindInfo) -> Result<(Self::Endpoint, Self::Incoming), Error>;
    fn shutdown(&self) -> Result<(), Error>;
}

trait State {}
struct Stopped;
impl State for Stopped {}
struct Running<E, I>
where
    E: Endpoint,
    I: Iterator<Item = Result<E::Connection, Error>>,
{
    endpoint: E,
    incoming: I,
}
impl<E, I> State for Running<E, I>
where
    E: Endpoint,
    I: Iterator<Item = Result<E::Connection, Error>>,
{
}

struct Protocol<T, St>
where
    St: State,
    T: Transport,
{
    transport: T,

    state: St,
}

impl<T, St> Protocol<T, St>
where
    St: State,
    T: Transport,
{
    fn new(transport: T) -> Protocol<T, Stopped> {
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
    fn start(self) -> Result<Protocol<T, Running<T::Endpoint, T::Incoming>>, Error> {
        let (endpoint, incoming) = self.transport.bind(BindInfo {
            addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0)),
            advertise_addrs: vec![],
            peer_id: "1234abcd".to_string(),
        })?;

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
    I: Iterator<Item = Result<E::Connection, Error>>,
{
    fn accept(&mut self) -> Result<Peer<E::Connection, peer::Connected>, Error> {
        let connection = self.state.incoming.next().unwrap()?;

        Ok(Peer::from(connection))
    }

    fn connect(&self) -> Result<Peer<E::Connection, peer::Connected>, Error> {
        let connection = self.state.endpoint.connect()?;

        Ok(Peer::from(connection))
    }

    fn stop(self) -> Result<Protocol<T, Stopped>, Error> {
        self.transport.shutdown()?;

        Ok(Protocol {
            transport: self.transport,
            state: Stopped {},
        })
    }
}
