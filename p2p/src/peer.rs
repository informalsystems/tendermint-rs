use std::marker::PhantomData;

enum Error {}
enum Message {}

trait AddressBook {}
trait Discovery {}

// TRANSPORT
trait Endpoint {
    type Peer;

    fn connect(&self) -> Result<Self::Peer, Error>;
}

trait Transport {
    type Peer: Peer;
    type Endpoint: Endpoint<Peer = <Self as Transport>::Peer>;
    type Incoming: Iterator<Item = Result<<<Self as Transport>::Endpoint as Endpoint>::Peer, Error>>;

    fn bind(&self) -> Result<(Self::Endpoint, Self::Incoming), Error>;
}

trait TransportState {}
struct Stopped;
impl TransportState for Stopped {}
struct Running<E, I>
where
    E: Endpoint,
    I: Iterator<Item = Result<E::Peer, Error>>,
{
    endpoint: E,
    incoming: I,
}
impl<E, I> TransportState for Running<E, I>
where
    E: Endpoint,
    I: Iterator<Item = Result<E::Peer, Error>>,
{
}

struct TransportProtocol<T, State>
where
    State: TransportState,
    T: Transport,
{
    transport: T,

    state: State,
}

impl<T, State> TransportProtocol<T, State>
where
    State: TransportState,
    T: Transport,
{
    fn new(transport: T) -> TransportProtocol<T, Stopped> {
        TransportProtocol {
            transport,
            state: Stopped,
        }
    }
}

impl<T> TransportProtocol<T, Stopped>
where
    T: Transport,
{
    fn start<I>(self) -> Result<TransportProtocol<T, Running<T::Endpoint, T::Incoming>>, Error> {
        let (endpoint, incoming) = self.transport.bind()?;

        Ok(TransportProtocol {
            transport: self.transport,
            state: Running { endpoint, incoming },
        })
    }
}

impl<T, E, I> TransportProtocol<T, Running<E, I>>
where
    T: Transport,
    E: Endpoint,
    E::Peer: Peer,
    I: Iterator<Item = Result<E::Peer, Error>>,
{
    fn accept(&mut self) -> Result<PeerProtocol<E::Peer, Connected>, Error> {
        let peer = self.state.incoming.next().unwrap()?;

        Ok(PeerProtocol {
            peer,
            _state: PhantomData,
        })
    }

    fn connect(&self) -> Result<PeerProtocol<E::Peer, Connected>, Error> {
        let peer = self.state.endpoint.connect()?;

        Ok(PeerProtocol {
            peer,
            _state: PhantomData,
        })
    }

    fn stop(self) -> Result<TransportProtocol<T, Stopped>, Error> {
        Ok(TransportProtocol {
            transport: self.transport,
            state: Stopped {},
        })
    }
}

// PEER
trait Peer {}

trait PeerState {}
enum Disconnected {}
impl PeerState for Disconnected {}
enum Connected {}
impl PeerState for Connected {}

struct PeerProtocol<P, State>
where
    P: Peer,
    State: PeerState,
{
    peer: P,

    _state: std::marker::PhantomData<State>,
}
