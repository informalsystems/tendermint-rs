use std::marker::PhantomData;

use crate::transport::Connection;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub enum Message {}

pub trait State {}
pub enum Disconnected {}
impl State for Disconnected {}
pub enum Connected {}
impl State for Connected {}

pub struct Peer<Conn, St>
where
    Conn: Connection,
    St: State,
{
    connection: Conn,

    _state: PhantomData<St>,
}

impl<Conn> From<Conn> for Peer<Conn, Connected>
where
    Conn: Connection,
{
    fn from(connection: Conn) -> Peer<Conn, Connected> {
        Peer {
            connection,
            _state: PhantomData,
        }
    }
}

impl<Conn> Iterator for Peer<Conn, Connected>
where
    Conn: Connection,
{
    type Item = Result<Message, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO(xla): This is the place where we read bytes from the underlying connection and
        // serialise them into `Message`s. If we can achieve that, that means no matter what
        // connection is plugged we guarantee proper handling of wire protocol. In turn that means
        // we assume the interface to be byte streams.
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
