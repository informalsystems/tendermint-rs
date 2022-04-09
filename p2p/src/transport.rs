//! Abstractions that describe types which support the physical transport - i.e. connection
//! management - used in the p2p stack.

use std::net::{SocketAddr, ToSocketAddrs};

use async_trait::async_trait;
use eyre::Result;
use futures::Stream;

use tendermint::node;
use tendermint::public_key::PublicKey;

/// Information which resources to bind to and how to identify on the network.
pub struct BindInfo<A>
where
    A: ToSocketAddrs,
{
    /// List of addresses to be communicated as publicly reachable to other nodes, which in turn
    /// can use that to share with third parties.
    ///
    /// TODO(xla): Depending on where this information is going to be disseminated it might be
    /// better placed in a higher-level protocol. What stands in opposition to that is the fact
    /// that advertised addresses will be helpful for hole punching and other involved network
    /// traversals.
    pub advertise_addrs: A,
    /// Local address(es) to bind to and accept connections on.
    pub bind_addrs: A,
    /// Public key of the peer used for identity on the network.
    pub public_key: PublicKey,
}

/// Information to establish a connection to a remote peer and validate its identity.
pub struct ConnectInfo<A>
where
    A: ToSocketAddrs,
{
    /// Known address(es) of the peer.
    pub addrs: A,
    /// The expected id of the remote peer.
    pub id: node::Id,
}

/// Known list of typed streams.
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum StreamId {
    /// Stream to exchange message concerning Peer Exchange.
    Pex,
}

/// Envelope to trace the original direction of an established connection.
pub enum Direction<Conn> {
    /// A peer that connected to the local node.
    Incoming(Conn),
    /// A remote peer the local node established a connection to.
    Outgoing(Conn),
}

/// Trait that describes the send end of a stream.
#[async_trait]
pub trait StreamSend: Send + Sync {
    /// Sends the message to the peer over the open stream. `msg` should be a valid and properly
    /// encoded byte array according to the supported messages of the stream.
    ///
    /// # Errors
    ///
    /// * If the underlying I/O operations fail.
    /// * If the stream is closed.
    /// * If the peer is gone
    async fn send<B: AsRef<[u8]>>(msg: B) -> Result<()>;
}

/// Trait which describes the core concept of a connection between two peers established by
/// `[Transport]`.
#[async_trait]
pub trait Connection: Send {
    /// Errors emitted by the connection.
    type Error;
    /// Read end of a bidirectional stream. Carries a finite stream of framed messages. Decoding is
    /// left to the caller and should correspond to the type of stream.
    type StreamRead: Stream<Item = Result<Vec<u8>>> + Send + Sync;
    /// Send end of a stream.
    type StreamSend: StreamSend;

    /// Returns the list of advertised addresses known for this connection.
    fn advertised_addrs(&self) -> Vec<SocketAddr>;
    /// Tears down the connection  and releases all attached resources.
    ///
    /// # Errors
    ///
    /// * If release of attached resources failed.
    async fn close(&self) -> Result<()>;
    /// Returns the local address for the connection.
    fn local_addr(&self) -> SocketAddr;
    /// Opens a new bi-bidirectional stream for the given [`StreamId`].
    ///
    /// # Errors
    ///
    /// * If the stream type is not supported.
    /// * If the peer is gone.
    /// * If resources necessary for the stream creation aren't available/accessible.
    async fn open_bidirectional(
        &self,
        stream_id: StreamId,
    ) -> Result<(Self::StreamRead, Self::StreamSend), Self::Error>;
    /// Public key of the remote peer.
    fn public_key(&self) -> PublicKey;
    /// Local address(es) to the endpoint listens on.
    fn remote_addr(&self) -> SocketAddr;
}

/// Local handle on a resource which allows connecting to remote peers.
#[async_trait]
pub trait Endpoint<A>: Send + Sync
where
    A: ToSocketAddrs,
{
    /// Core type that represents a connection between two peers established through the transport.
    type Connection;

    /// Establishes a new connection to a remote peer.
    ///
    /// # Errors
    ///
    /// * If the remote is not reachable.
    /// * If resources necessary for the connection creation aren't available/accessible.
    async fn connect(&self, info: ConnectInfo<A>) -> Result<Self::Connection>;
    /// Local address(es) the endpoint listens on.
    fn listen_addrs(&self) -> Vec<SocketAddr>;
}

/// Trait that describes types which support connection management of the p2p stack.
#[async_trait]
pub trait Transport<A>
where
    A: ToSocketAddrs,
{
    /// Core type that represents a connection between two peers established through the transport.
    type Connection: Connection;
    /// Local handle on a resource which allows connecting to remote peers.
    type Endpoint: Endpoint<A, Connection = <Self as Transport<A>>::Connection> + Drop;
    /// Infinite stream of inbound connections.
    type Incoming: Stream<Item = Result<<Self as Transport<A>>::Connection>> + Send + Sync;

    /// Consumes the transport to bind the resources in exchange for the `Endpoint` and `Incoming`
    /// stream.
    ///
    /// # Errors
    ///
    /// * If resource allocation fails for lack of privileges or being not available.
    async fn bind(self, bind_info: BindInfo<A>) -> Result<(Self::Endpoint, Self::Incoming)>;
}
