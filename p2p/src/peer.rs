use std::collections::HashMap;
use std::convert::TryFrom;
use std::thread;

use eyre::{eyre, Report, Result, WrapErr};
use flume::{self, Receiver, Sender};

use tendermint::node;

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

pub struct Peer<St>
where
    St: private::Sealed,
{
    pub id: node::Id,
    pub state: St,
}

impl<Conn> TryFrom<Direction<Conn>> for Peer<Connected<Conn>>
where
    Conn: Connection,
{
    type Error = Report;

    fn try_from(connection: Direction<Conn>) -> Result<Peer<Connected<Conn>>, Self::Error> {
        let pk = match &connection {
            Direction::Incoming(conn) | Direction::Outgoing(conn) => conn.public_key(),
        };

        let id =
            node::Id::try_from(pk).map_err(|err| eyre!("unabel to obtain id, got {:?}", err))?;

        Ok(Peer {
            id,
            state: Connected { connection },
        })
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
