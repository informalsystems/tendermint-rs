use std::collections::HashMap;

use eyre::Result;
use flume::{Receiver, Sender};

use crate::transport::{Connection, Direction, StreamId};

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub enum Message {}

struct Stream {
    recv: Receiver<Message>,
    send: Sender<Message>,
}

pub trait State: private::Sealed {}

pub struct Disconnected<Conn> {
    connection: Direction<Conn>,
}
impl<Conn> State for Disconnected<Conn> {}

pub struct Connected<Conn> {
    connection: Direction<Conn>,
}
impl<Conn> State for Connected<Conn> {}

pub struct Running<Conn> {
    connection: Direction<Conn>,
}
impl<Conn> State for Running<Conn> {}

pub struct Stopped {
    error: Option<Error>,
}
impl State for Stopped {}

pub struct Peer<St>
where
    St: State,
{
    state: St,
}

impl<Conn> From<Direction<Conn>> for Peer<Connected<Conn>> {
    fn from(connection: Direction<Conn>) -> Peer<Connected<Conn>> {
        Peer {
            state: Connected { connection },
        }
    }
}

impl<Conn> Peer<Connected<Conn>>
where
    Conn: Connection,
{
    fn run(self, stream_ids: Vec<StreamId>) -> Result<Peer<Running<Conn>>> {
        Ok(Peer {
            state: Running {
                connection: self.state.connection,
            },
        })
    }

    fn stop(self) -> Result<Peer<Stopped>> {
        match self.state.connection {
            Direction::Incoming(conn) | Direction::Outgoing(conn) => conn.close()?,
        }

        Ok(Peer {
            state: Stopped { error: None },
        })
    }
}

impl<Conn> Peer<Running<Conn>>
where
    Conn: Connection,
{
    fn send(&self, message: Message) -> Result<()> {
        // TODO(xla): Map message to stream id.
        todo!()
    }

    fn stop(self) -> Result<Peer<Stopped>> {
        match self.state.connection {
            Direction::Incoming(conn) | Direction::Outgoing(conn) => conn.close()?,
        }

        Ok(Peer {
            state: Stopped { error: None },
        })
    }
}

impl<Conn> Iterator for Peer<Running<Conn>> {
    type Item = Result<Message, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO(xla): This is the place where we read bytes from the underlying connection and
        // serialise them into `Message`s. If we can achieve that, that means no matter what
        // connection is plugged we guarantee proper handling of the wire protocol. In turn that
        // means we assume the interface to be byte streams.
        //
        // Assumption here is that a unified event/message stream is wanted. An alternative model
        // would be to have specialised streams where the messages are typed, similar to the idea
        // of channelIded packets in the existing MConn.
        //
        // To decide what the right surface area is, there needs to be some exploration into the
        // upper layer of peer management and coordination (Supervisor?). Which should inform if
        // there is a need for finer grained control.

        None
    }
}

mod private {
    use super::{Connected, Disconnected, Running, Stopped};

    /// Constraint for [sealed traits] under the `transport` module hierarchy.
    ///
    /// [sealed traits]: https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
    pub trait Sealed {}

    impl<Conn> Sealed for Connected<Conn> {}
    impl<Conn> Sealed for Disconnected<Conn> {}
    impl<Conn> Sealed for Running<Conn> {}
    impl Sealed for Stopped {}
}
