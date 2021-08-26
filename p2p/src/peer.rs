#![allow(clippy::use_self)]

use std::collections::HashMap;
use std::convert::TryFrom;
use std::thread;

use eyre::{eyre, Report, Result};
use flume::{self, Receiver, Sender};

use tendermint::node;

use crate::message;
use crate::transport::{Connection, Direction, StreamId, StreamSend as _};

pub trait State: private::Sealed {}

pub struct Connected<Conn> {
    connection: Direction<Conn>,
}
impl<Conn> State for Connected<Conn> {}

pub struct Running<Conn> {
    connection: Direction<Conn>,
    pub receiver: Receiver<message::Receive>,
    // senders: HashMap<StreamId, Sender<Box<dyn message::Outgoing>>>,
    senders: HashMap<StreamId, Sender<message::Send>>,
}
impl<Conn> State for Running<Conn> {}

pub struct Stopped {
    pub error: Option<Report>,
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

    fn try_from(connection: Direction<Conn>) -> Result<Self, Self::Error> {
        let pk = match &connection {
            Direction::Incoming(conn) | Direction::Outgoing(conn) => conn.public_key(),
        };

        let id = node::Id::try_from(pk)
            .map_err(|err| eyre!("unable to obtain node id, got {:?}", err))?;

        Ok(Self {
            id,
            state: Connected { connection },
        })
    }
}

impl<Conn> Peer<Connected<Conn>>
where
    Conn: Connection,
{
    pub fn run(self, stream_ids: Vec<StreamId>) -> Result<Peer<Running<Conn>>>
    where
        <Conn as Connection>::StreamReceive: 'static,
        <Conn as Connection>::StreamSend: 'static,
    {
        // FIXME(xla): No queue should be unbounded, backpressure should be finley tuned and/or
        // tunable.
        let (receive_tx, receiver) = flume::unbounded::<message::Receive>();
        let mut senders = HashMap::new();

        for stream_id in stream_ids {
            let (mut stream_receive, stream_send) = match &self.state.connection {
                Direction::Incoming(conn) | Direction::Outgoing(conn) => {
                    conn.open_bidirectional(stream_id)?
                }
            };

            {
                let tx = receive_tx.clone();

                thread::spawn(move || {
                    for res in stream_receive.next() {
                        match res {
                            Err(_err) => todo!(),
                            Ok(buf) => match message::Receive::decode(stream_id, buf) {
                                Err(_err) => todo!(),
                                Ok(msg) => match tx.try_send(msg) {
                                    Ok(_) => continue,
                                    // The receiver is gone, this subroutine needs to be terminated.
                                    Err(flume::TrySendError::Disconnected(_)) => break,
                                    // TODO(xla): Graceful handling needs to be figured out.
                                    Err(flume::TrySendError::Full(_)) => todo!(),
                                },
                            },
                        }
                    }

                    // TODO(xla): Log subroutine termination.
                });
            }

            // FIXME(xla): No queue should be unbounded, backpressure should be finley tuned and/or
            // tunable.
            let (send_tx, send_rx) = flume::unbounded::<message::Send>();

            thread::spawn(move || {
                loop {
                    match send_rx.recv() {
                        // If the sender is gone this subroutine needs to vanish with it.
                        Err(flume::RecvError::Disconnected) => break,
                        Ok(msg) => {
                            let mut buf = vec![];

                            match msg.encode(&mut buf) {
                                Err(_err) => todo!(),
                                Ok(()) => match stream_send.send(&buf) {
                                    Err(_err) => todo!(),
                                    Ok(_) => continue,
                                },
                            }
                        }
                    }
                }

                // TODO(xla): Log subroutine termination.
            });

            senders.insert(stream_id, send_tx);
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
}

impl<Conn> Peer<Running<Conn>>
where
    Conn: Connection,
{
    pub fn send(&self, msg: message::Send) -> Result<()> {
        let id = match &msg {
            message::Send::Pex(_msg) => StreamId::Pex,
        };

        match self.state.senders.get(&id) {
            Some(sender) => sender.send(msg).map_err(Report::from),
            None => Err(Report::msg("no open stream to send on")),
        }
    }

    pub fn stop(self) -> Peer<Stopped> {
        let error = match self.state.connection {
            Direction::Incoming(conn) | Direction::Outgoing(conn) => conn.close().err(),
        };

        Peer {
            id: self.id,
            state: Stopped { error },
        }
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
