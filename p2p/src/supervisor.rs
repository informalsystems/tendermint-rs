use crate::peer::{self, Peer};
use crate::transport::{self, Connection, Endpoint, Transport};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Transport(#[from] transport::Error),
}

trait State {}
struct Stopped;
impl State for Stopped {}
struct Running<E, I>
where
    E: Endpoint,
    I: Iterator<Item = Result<E::Connection, transport::Error>>,
{
    endpoint: E,
    incoming: I,
}
impl<E, I> State for Running<E, I>
where
    E: Endpoint,
    I: Iterator<Item = Result<E::Connection, transport::Error>>,
{
}

struct Supervisor<T, St>
where
    St: State,
    T: Transport,
{
    transport: T,

    state: St,
}

impl<T, St> Supervisor<T, St>
where
    St: State,
    T: Transport,
{
    fn new(transport: T) -> Supervisor<T, Stopped> {
        Supervisor {
            transport,
            state: Stopped,
        }
    }
}

impl<T> Supervisor<T, Stopped>
where
    T: Transport,
{
    fn start(self) -> Result<Supervisor<T, Running<T::Endpoint, T::Incoming>>, Error> {
        let (endpoint, incoming) = self.transport.bind()?;

        Ok(Supervisor {
            transport: self.transport,
            state: Running { endpoint, incoming },
        })
    }
}

impl<T, E, I> Supervisor<T, Running<E, I>>
where
    T: Transport,
    E: Endpoint,
    E::Connection: Connection,
    I: Iterator<Item = Result<E::Connection, transport::Error>>,
{
    fn accept(&mut self) -> Result<Peer<E::Connection, peer::Connected>, Error> {
        let connection = self.state.incoming.next().unwrap()?;

        Ok(Peer::from(connection))
    }

    fn connect(&self) -> Result<Peer<E::Connection, peer::Connected>, Error> {
        let connection = self.state.endpoint.connect()?;

        Ok(Peer::from(connection))
    }

    fn stop(self) -> Result<Supervisor<T, Stopped>, Error> {
        Ok(Supervisor {
            transport: self.transport,
            state: Stopped {},
        })
    }
}
