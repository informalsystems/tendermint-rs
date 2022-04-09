use eyre::{eyre, Report, Result};

use tendermint::node;

use crate::transport::{Connection, Direction, StreamId};

pub trait State: private::Sealed {}

pub struct Connected<C> {
    connection: Direction<C>,
}
impl<C> State for Connected<C> {}

pub struct Running<C> {
    connection: Direction<C>,
}
impl<C> State for Running<C> {}

pub struct Stopped;
impl State for Stopped {}

pub struct Peer<St> {
    pub id: node::Id,
    pub state: St,
}

impl<C> TryFrom<Direction<C>> for Peer<Connected<C>>
where
    C: Connection,
{
    type Error = Report;

    fn try_from(connection: Direction<C>) -> Result<Self, Self::Error> {
        let pk = match &connection {
            Direction::Incoming(c) | Direction::Outgoing(c) => c.public_key(),
        };

        let id =
            node::Id::try_from(pk).map_err(|err| eyre!("failed to obtain node id: {:?}", err))?;

        Ok(Self {
            id,
            state: Connected { connection },
        })
    }
}

impl<C> Peer<Connected<C>>
where
    C: Connection,
{
    pub async fn run(self, stream_ids: Vec<StreamId>) -> Result<Peer<Running<C>>> {
        for id in stream_ids {
            self.open_bidirectional(id).await;
        }

        Ok(Peer {
            id: self.id,
            state: Running {
                connection: self.state.connection,
            },
        })
    }

    async fn open_bidirectional(
        &self,
        id: StreamId,
    ) -> Result<(C::StreamRead, C::StreamSend), C::Error> {
        match self.state.connection {
            Direction::Incoming(ref c) | Direction::Outgoing(ref c) => {
                c.open_bidirectional(id).await
            }
        }
    }
}

impl<C> Peer<Running<C>>
where
    C: Connection,
{
    pub async fn stop(self) -> Result<Peer<Stopped>> {
        match self.state.connection {
            Direction::Incoming(ref c) | Direction::Outgoing(ref c) => {
                c.close().await?;
            }
        }

        Ok(Peer {
            id: self.id,
            state: Stopped,
        })
    }
}

mod private {
    use super::{Connected, Running, Stopped};

    /// Constraint for [sealed traits] under the `transport` module hierarchy.
    ///
    /// [sealed traits]: https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
    pub trait Sealed {}

    impl<C> Sealed for Connected<C> {}
    impl<C> Sealed for Running<C> {}
    impl Sealed for Stopped {}
}
