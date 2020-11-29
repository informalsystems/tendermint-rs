#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub trait Connection: Clone {
    type Stream;

    fn open(&self) -> Result<Self::Stream, Error>;
}

pub trait Endpoint {
    type Connection;

    fn connect(&self) -> Result<Self::Connection, Error>;
}

pub trait Transport {
    type Connection: Connection;
    type Endpoint: Endpoint<Connection = <Self as Transport>::Connection>;
    type Incoming: Iterator<Item = Result<<Self as Transport>::Connection, Error>>;

    fn bind(&self) -> Result<(Self::Endpoint, Self::Incoming), Error>;
}
