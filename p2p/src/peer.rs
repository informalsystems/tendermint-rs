use std::collections::HashMap;

use eyre::Result;

use crate::transport::{Connection, StreamId};

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub enum Message {}

pub trait State: private::Sealed {}

pub struct Disconnected {}
impl State for Disconnected {}

pub struct Connected {}
impl State for Connected {}

pub struct Running<S> {
    streams: HashMap<StreamId, S>,
}
impl<S> State for Running<S> {}

pub struct Stopped {
    error: Option<Error>,
}
impl State for Stopped {}

pub struct Peer<Conn, St>
where
    St: State,
{
    connection: Conn,

    _state: St,
}

impl<Conn> From<Conn> for Peer<Conn, Connected> {
    fn from(connection: Conn) -> Peer<Conn, Connected> {
        Peer {
            connection,

            _state: Connected {},
        }
    }
}

impl<Conn> Peer<Conn, Connected>
where
    Conn: Connection,
{
    fn run(
        self,
        stream_ids: Vec<StreamId>,
    ) -> Result<Peer<Conn, Running<<Conn as Connection>::Stream>>> {
        let streams = HashMap::new();

        for id in &stream_ids {
            let stream = self.connection.open_bidirectional(id)?;
        }

        Ok(Peer {
            connection: self.connection,

            _state: Running { streams },
        })
    }

    fn stop(self) -> Result<Peer<Conn, Stopped>> {
        self.connection.close()?;

        Ok(Peer {
            connection: self.connection,

            _state: Stopped { error: None },
        })
    }
}

impl<Conn> Peer<Conn, Running<<Conn as Connection>::Stream>>
where
    Conn: Connection,
{
    fn send(&self, message: Message) -> Result<()> {
        // TODO(xla): Map message to stream id.
        todo!()
    }

    fn stop(self) -> Result<Peer<Conn, Stopped>> {
        self.connection.close()?;

        Ok(Peer {
            connection: self.connection,

            _state: Stopped { error: None },
        })
    }
}

impl<Conn> Iterator for Peer<Conn, Running<<Conn as Connection>::Stream>>
where
    Conn: Connection,
{
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

    impl Sealed for Connected {}
    impl Sealed for Disconnected {}
    impl<S> Sealed for Running<S> {}
    impl Sealed for Stopped {}
}
