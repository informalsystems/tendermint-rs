use std::marker::PhantomData;

use crate::transport::Connection;

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
