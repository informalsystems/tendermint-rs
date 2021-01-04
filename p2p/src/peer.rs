use std::collections::HashMap;
use std::thread;

use eyre::Result;
use flume::{self, Receiver, Sender};

use tendermint::node;
use tendermint::public_key::PublicKey;

use crate::message;
use crate::transport::{Connection, Direction, StreamId};

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub trait State: private::Sealed {}

pub struct Connected<Conn> {
    connection: Direction<Conn>,
}
impl<Conn> State for Connected<Conn> {}

pub struct Running<Conn> {
    connection: Direction<Conn>,
    pub receiver: Receiver<message::Receive>,
    senders: HashMap<StreamId, Sender<Box<dyn message::Outgoing>>>,
}
impl<Conn> State for Running<Conn> {}

pub struct Stopped {
    error: Option<Error>,
}
impl State for Stopped {}

pub struct Peer<St> {
    pub id: node::Id,

    pub state: St,
}

impl<Conn> From<Direction<Conn>> for Peer<Connected<Conn>>
where
    Conn: Connection,
{
    fn from(connection: Direction<Conn>) -> Peer<Connected<Conn>> {
        let pk = match &connection {
            Direction::Incoming(conn) => conn.public_key(),
            Direction::Outgoing(conn) => conn.public_key(),
        };

        let id = match pk {
            PublicKey::Ed25519(ed25519) => node::Id::from(ed25519),
            _ => panic!(),
        };

        Peer {
            id,

            state: Connected { connection },
        }
    }
}

impl<Conn> Peer<Connected<Conn>>
where
    Conn: Connection,
{
    pub fn run(self, stream_ids: Vec<StreamId>) -> Result<Peer<Running<Conn>>> {
        // FIXME(xla): No queue should be unbounded, backpressure should be finley tuned and/or
        // tunable.
        let (receive_tx, receiver) = flume::unbounded::<message::Receive>();
        let mut senders = HashMap::new();

        for stream_id in &stream_ids {
            let (read, write) = match &self.state.connection {
                Direction::Incoming(conn) | Direction::Outgoing(conn) => {
                    conn.open_bidirectional(stream_id)?
                }
            };

            {
                let tx = receive_tx.clone();

                thread::spawn(move || {
                    loop {
                        // Read bytes
                        // Deserialize into typed message
                        // send on receiver_send

                        // End the read loop when all receivers are gone.
                        if tx
                            .send(message::Receive::Pex(message::PexReceive::Noop))
                            .is_err()
                        {
                            break;
                        }
                    }
                });
            }

            let (write_tx, write_rx) = flume::unbounded();

            thread::spawn(move || {
                loop {
                    let res = write_rx.recv();

                    // The only error possible is a disconnection.
                    if res.is_err() {
                        break;
                    }

                    let msg = res.unwrap();

                    // Serialise message
                    // write bytes
                }
            });

            senders.insert(*stream_id, write_tx);
        }

        Ok(Peer {
            id: self.id,

            state: Running {
                connection: self.state.connection,
                receiver,
                senders,
            },
        })
    }

    fn stop(self) -> Result<Peer<Stopped>> {
        match self.state.connection {
            Direction::Incoming(conn) | Direction::Outgoing(conn) => conn.close()?,
        }

        Ok(Peer {
            id: self.id,

            state: Stopped { error: None },
        })
    }
}

impl<Conn> Peer<Running<Conn>>
where
    Conn: Connection,
{
    pub fn send(&self, message: message::Send) -> Result<()> {
        // TODO(xla): Map message to stream id.
        todo!()
    }

    pub fn stop(self) -> Result<Peer<Stopped>> {
        match self.state.connection {
            Direction::Incoming(conn) | Direction::Outgoing(conn) => conn.close()?,
        }

        Ok(Peer {
            id: self.id,

            state: Stopped { error: None },
        })
    }
}

mod private {
    use super::{Connected, Running, Stopped};

    /// Constraint for [sealed traits] under the `transport` module hierarchy.
    ///
    /// [sealed traits]: https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
    pub trait Sealed {}

    impl<Conn> Sealed for Connected<Conn> {}
    impl<Conn> Sealed for Running<Conn> {}
    impl Sealed for Stopped {}
}
