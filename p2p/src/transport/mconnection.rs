//! `MConnection`: a Transport which multiplexes messages from different sources over a single TCP
//! stream.
//! Spec: https://github.com/tendermint/spec/blob/master/spec/p2p/connection.md#p2p-multiplex-connection

use std::collections::HashMap;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use ed25519_dalek as ed25519;
use eyre::{Result, WrapErr};
use flume::{self, Receiver, Sender};
use prost::Message as _;

use tendermint_proto::p2p::PacketMsg;

use crate::secret_connection::{self, SecretConnection, Version};
use crate::transport::{
    BindInfo, ConnectInfo, Connection, Endpoint, PublicKey, Read, StreamId, Transport, Write,
};

/// A Transport which multiplexes messages from different sources over a single TCP stream.
/// Spec: <https://github.com/tendermint/spec/blob/master/spec/p2p/connection.md#p2p-multiplex-connection>
pub struct MConnectionTransport {}

/// A `TcpStream` wrapped in `SecretConnection`.
pub struct MConnection {
    public_key: ed25519::PublicKey,
    // stream clone for shutting down connection
    stream: TcpStream,
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
    // clonable sender to write data to secret connection
    // see rw_loop where writing happens
    sender: Sender<PacketMsg>,
    // stream ID mapped to a sender
    // each `ReadVirtualStream` has a matching receiver
    streams: Arc<RwLock<HashMap<StreamId, Sender<PacketMsg>>>>,
}

/// An `Endpoint` for connecting to other peers.
pub struct MEndpoint {
    private_key: Arc<ed25519::Keypair>,
    protocol_version: Version,
    listen_addrs: Vec<SocketAddr>,
}

/// An `Iterator` for accepting connections.
pub struct MIncoming {
    tcp_listener: TcpListener,
    private_key: Arc<ed25519::Keypair>,
    protocol_version: Version,
}

// `MConnection` does not have independent streams, so we create a virtual one. It acts as a filter
// that only proxies messages for given stream ID.

/// Write part of the virtual stream.
pub struct WriteVirtualStream {
    stream_id: StreamId,
    sender: Sender<PacketMsg>,
}

/// Read part of the virtual stream.
pub struct ReadVirtualStream {
    receiver: Receiver<PacketMsg>,
}

impl MConnection {
    /// Opens a TCP connection to a remote host and establishes a secret connection
    /// [`SecretConnection`].
    fn connect(
        addr: &SocketAddr,
        private_key: &ed25519::Keypair,
        protocol_version: Version,
    ) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;

        let stream_clone = stream.try_clone()?;
        let local_addr = stream.local_addr()?;
        let peer_addr = stream.peer_addr()?;

        let public_key = private_key.public;
        let secret_connection = SecretConnection::new(stream, private_key, protocol_version)?;

        let (sender, loop_receiver) = flume::bounded(0);
        let streams = Arc::new(RwLock::new(HashMap::new()));

        let streams_clone = streams.clone();
        std::thread::spawn(move || {
            MConnection::rw_loop(secret_connection, loop_receiver, streams_clone)
        });

        Ok(Self {
            public_key,
            stream: stream_clone,
            local_addr,
            peer_addr,
            sender,
            streams,
        })
    }

    /// Opens a TCP connection to a remote host with a timeout and establishes a secret connection
    /// [`SecretConnection`].
    fn connect_timeout(
        addr: &SocketAddr,
        timeout: Duration,
        private_key: &ed25519::Keypair,
        protocol_version: Version,
    ) -> Result<Self> {
        let stream = TcpStream::connect_timeout(addr, timeout)?;

        let stream_clone = stream.try_clone()?;
        let local_addr = stream.local_addr()?;
        let peer_addr = stream.peer_addr()?;

        let public_key = private_key.public;
        let secret_connection = SecretConnection::new(stream, private_key, protocol_version)?;

        let (sender, loop_receiver) = flume::bounded(0);
        let streams = Arc::new(RwLock::new(HashMap::new()));

        let streams_clone = streams.clone();
        std::thread::spawn(move || {
            MConnection::rw_loop(secret_connection, loop_receiver, streams_clone)
        });

        Ok(Self {
            public_key,
            stream: stream_clone,
            local_addr,
            peer_addr,
            sender,
            streams,
        })
    }

    fn rw_loop(
        mut secret_connection: SecretConnection<TcpStream>,
        rx: Receiver<PacketMsg>,
        streams: Arc<RwLock<HashMap<StreamId, Sender<PacketMsg>>>>,
    ) {
        let mut read_buf = Vec::with_capacity(8096);

        loop {
            // If any stream is trying to send a message, go for it.
            if let Ok(msg) = rx.try_recv() {
                let mut buf = Vec::new();
                msg.encode_length_delimited(&mut buf)
                    .expect("encode to always succeed");
                if let Err(e) = secret_connection.write_all(&buf) {
                    println!("can't write to connection: {}. exiting...", e);
                    return;
                }
            }

            // If there's a new incoming message, send it to all streams.
            let mut buf = Vec::with_capacity(secret_connection::DATA_MAX_SIZE);
            match secret_connection.read(&mut buf) {
                Ok(_) => {
                    read_buf.append(&mut buf);
                    if let Ok(msg) = PacketMsg::decode_length_delimited(&*read_buf) {
                        read_buf.clear();
                        if let Ok(streams) = streams.read() {
                            let stream_id = match msg.channel_id {
                                0 => StreamId::Pex,
                                _ => {
                                    println!("unknown channel_id: {}", msg.channel_id);
                                    continue;
                                }
                            };
                            if let Some(s) = streams.get(&stream_id) {
                                if let Err(e) = s.send(msg) {
                                    println!("can't relay msg: {}", e)
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("can't read from connection: {}. exiting...", e);
                    return;
                }
            }
        }
    }
}

impl Connection for MConnection {
    type Error = std::io::Error;
    type Read = ReadVirtualStream;
    type Write = WriteVirtualStream;

    fn close(&self) -> Result<()> {
        self.stream
            .shutdown(Shutdown::Both)
            .wrap_err("failed to shutdown")
    }

    fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    fn remote_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    fn open_bidirectional(
        &self,
        stream_id: StreamId,
    ) -> Result<(Self::Read, Self::Write), Self::Error> {
        let (sender, receiver) = flume::bounded(0);
        let mut streams = self
            .streams
            .write()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        if streams.contains_key(&stream_id) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "stream already exists",
            ));
        }
        streams.insert(stream_id, sender);

        Ok((
            ReadVirtualStream { receiver },
            WriteVirtualStream {
                stream_id,
                sender: self.sender.clone(),
            },
        ))
    }

    fn public_key(&self) -> PublicKey {
        tendermint::PublicKey::Ed25519(self.public_key)
    }
}

impl Endpoint for MEndpoint {
    type Connection = MConnection;

    /// Connects to the specified address using either `MConnection::connect` or
    /// `MConnection::connect_timeout` depending on whenever `ConnectInfo.timeout` is zero or not.
    fn connect(&self, info: ConnectInfo) -> Result<Self::Connection> {
        if info.timeout > Duration::new(0, 0) {
            MConnection::connect_timeout(
                &info.addr,
                info.timeout,
                &self.private_key,
                self.protocol_version,
            )
        } else {
            MConnection::connect(&info.addr, &self.private_key, self.protocol_version)
        }
    }

    fn listen_addrs(&self) -> Vec<SocketAddr> {
        self.listen_addrs.clone()
    }
}

impl Iterator for MIncoming {
    type Item = Result<MConnection>;

    /// Advances the iterator and returns the next `MConnection`.
    fn next(&mut self) -> Option<Result<MConnection>> {
        let public_key = self.private_key.public;

        match self
            .tcp_listener
            .incoming()
            .next()
            .expect("Incoming to always return Some") // it's safe to unwrap here because Incoming never returns None
            .wrap_err("failed to accept conn")
        {
            Ok(stream) => {
                let stream_clone = stream.try_clone().wrap_err("failed to clone stream");
                let local_addr = stream.local_addr().wrap_err("failed to get local addr");
                let peer_addr = stream.peer_addr().wrap_err("failed to get peer addr");

                let (sender, loop_receiver) = flume::bounded(0);
                let streams = Arc::new(RwLock::new(HashMap::new()));
                let streams_clone = streams.clone();

                match (
                    SecretConnection::new(stream, &self.private_key, self.protocol_version),
                    stream_clone,
                    local_addr,
                    peer_addr,
                ) {
                    (Ok(secret_connection), Ok(stream_clone), Ok(local_addr), Ok(peer_addr)) => {
                        let conn = MConnection {
                            public_key,
                            stream: stream_clone,
                            local_addr,
                            peer_addr,
                            sender,
                            streams,
                        };

                        std::thread::spawn(move || {
                            MConnection::rw_loop(secret_connection, loop_receiver, streams_clone)
                        });

                        Some(Ok(conn))
                    }
                    (Err(e), _, _, _)
                    | (_, Err(e), _, _)
                    | (_, _, Err(e), _)
                    | (_, _, _, Err(e)) => Some(Err(e)),
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

impl Transport for MConnectionTransport {
    type Connection = MConnection;
    type Endpoint = MEndpoint;
    type Incoming = MIncoming;

    /// Creates a new `TcpListener` which will be bound to the specified address.
    /// The private key will be used to establish a `SecretConnection` each time you connect or
    /// accept a connection.
    ///
    /// See `TcpListener::bind`
    ///
    /// # Examples
    ///
    /// Creates a TCP listener bound to `127.0.0.1:8080`:
    ///
    /// ```no_run
    /// use rand_core::OsRng;
    /// use ed25519_dalek::Keypair;
    ///
    /// use crate::tendermint_p2p::transport::{BindInfo, Transport};
    /// use crate::tendermint_p2p::transport::mconnection::MConnectionTransport;
    ///
    /// let mut csprng = OsRng{};
    ///
    /// let t = MConnectionTransport{};
    /// let (endpoint, incoming) = t.bind(BindInfo{
    ///     addr: "127.0.0.1:8080".parse().expect("valid addr"),
    ///     listen_addrs: vec!["192.168.1.2:26656".parse().expect("valid addr")],
    ///     private_key: Keypair::generate(&mut csprng),
    ///
    /// }).expect("bind to succeed");
    /// ```
    fn bind(self, bind_info: BindInfo) -> Result<(MEndpoint, MIncoming)> {
        let listener = TcpListener::bind(bind_info.addr)?;
        let pk = Arc::new(bind_info.private_key);
        Ok((
            MEndpoint {
                private_key: pk.clone(),
                // TODO: version should be determined based on the `tendermint` crate version
                // if we want to support old chains that are using Amino encoding
                // if not, we should just hardcode the value
                protocol_version: Version::V0_34,
                listen_addrs: bind_info.listen_addrs,
            },
            MIncoming {
                tcp_listener: listener,
                private_key: pk,
                // TODO: version should be determined based on the `tendermint` crate version
                // if we want to support old chains that are using Amino encoding
                // if not, we should just hardcode the value
                protocol_version: Version::V0_34,
            },
        ))
    }

    /// Noop.
    fn shutdown(&self) -> Result<()> {
        // The socket will be closed when the MConnectionTransport is dropped.
        Ok(())
    }
}

impl Read for ReadVirtualStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let msg = self
            .receiver
            .recv()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        buf.copy_from_slice(&msg.data);
        return Ok(msg.data.len());
    }
}

impl Write for WriteVirtualStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let msg = PacketMsg {
            channel_id: self.stream_id as i32,
            eof: false,
            data: buf.to_vec(),
        };

        let n = buf.len();
        self.sender
            .send(msg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::Keypair;
    use rand_core::OsRng;

    use std::io;
    use std::thread;

    use super::*;

    #[test]
    fn bind_and_connect() {
        let addr: SocketAddr;
        {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind to succeed");
            addr = listener.local_addr().expect("local_addr to be there");
        }
        let addr_copy = addr;

        let peer1 = thread::spawn(move || {
            let mut csprng = OsRng {};
            let t = MConnectionTransport {};
            let (_endpoint1, mut incoming1) = t
                .bind(BindInfo {
                    addr,
                    private_key: Keypair::generate(&mut csprng),
                    listen_addrs: vec!["192.168.1.1:26656".parse().expect("valid addr")],
                })
                .expect("bind to succeed");

            let c = incoming1.next();
            assert!(c.is_some());

            if let Some(Ok(c)) = c {
                let (mut r, _w) = c.open_bidirectional(StreamId::Pex).unwrap();

                let mut buf = Vec::new();
                let n = r.read(&mut buf);
                assert!(n.is_ok());
                assert_eq!(5, n.unwrap());
            }
        });

        let peer2 = thread::spawn(move || {
            let mut csprng = OsRng {};
            let t = MConnectionTransport {};

            let (endpoint2, _incoming2) = t
                .bind(BindInfo {
                    addr: "127.0.0.1:0".parse().expect("valid addr"),
                    private_key: Keypair::generate(&mut csprng),
                    listen_addrs: vec!["192.168.1.2:26656".parse().expect("valid addr")],
                })
                .expect("bind to succeed");

            let conn = endpoint2
                .connect(ConnectInfo {
                    addr: addr_copy,
                    timeout: Duration::new(0, 0),
                })
                .expect("bind to succeed");

            let (_r, mut w) = conn.open_bidirectional(StreamId::Pex).unwrap();
            let n = w.write(&[0, 0, 0, 0, 0]);
            assert!(n.is_ok());
            assert_eq!(5, n.unwrap());
        });

        peer1.join().expect("peer1 thread has panicked");
        peer2.join().expect("peer2 thread has panicked");
    }

    #[test]
    fn bind_and_connect_timeout() {
        // this IP is unroutable, so connections should always time out.
        let addr = "10.255.255.1:80".parse().expect("valid addr");

        let mut csprng = OsRng {};
        let t = MConnectionTransport {};

        let (endpoint2, _incoming2) = t
            .bind(BindInfo {
                addr: "127.0.0.1:0".parse().expect("valid addr"),
                private_key: Keypair::generate(&mut csprng),
                listen_addrs: vec!["192.168.1.2:26656".parse().expect("valid addr")],
            })
            .expect("bind to succeed");

        match endpoint2.connect(ConnectInfo {
            addr,
            timeout: Duration::new(1, 0),
        }) {
            Ok(_) => panic!("connection was successfull? Ooo"),
            Err(e) => {
                assert_eq!(
                    e.downcast_ref::<io::Error>()
                        .expect("err to be of instance of io::Error")
                        .kind(),
                    io::ErrorKind::TimedOut
                );
            }
        }
    }
}
