use std::convert::TryFrom;

use bytes::buf::{Buf, BufMut};
use eyre::{Report, Result};
use prost::Message as _;

use proto::Protobuf;
use tendermint_proto as proto;

use crate::transport::StreamId;

pub trait Outgoing: std::marker::Send + Sync {}

pub enum Receive {
    Pex(Pex),
}

impl From<proto::p2p::message::Sum> for Receive {
    fn from(raw: proto::p2p::message::Sum) -> Self {
        match raw {
            proto::p2p::message::Sum::PexAddrs(proto::p2p::PexAddrs { addrs }) => {
                Self::Pex(Pex::Addrs(addrs))
            }
            proto::p2p::message::Sum::PexRequest(_) => Self::Pex(Pex::Request),
        }
    }
}

impl Receive {
    pub fn decode<B>(stream_id: StreamId, buf: B) -> Result<Self>
    where
        B: Buf,
    {
        let raw = match stream_id {
            StreamId::Pex => match proto::p2p::Message::decode(buf) {
                Ok(out) => match out.sum {
                    Some(sum) => sum,
                    None => todo!(),
                },
                Err(_err) => todo!(),
            },
        };

        Ok(Self::from(raw))
    }
}

#[derive(Clone)]
pub enum Pex {
    Addrs(Vec<proto::p2p::NetAddress>),
    Request,
}

impl Outgoing for Pex {}

impl Protobuf<proto::p2p::Message> for Pex {}

impl TryFrom<proto::p2p::Message> for Pex {
    type Error = Report;

    fn try_from(proto::p2p::Message { sum }: proto::p2p::Message) -> Result<Self, Self::Error> {
        match sum {
            Some(proto::p2p::message::Sum::PexAddrs(proto::p2p::PexAddrs { addrs })) => {
                Ok(Self::Addrs(addrs))
            }
            Some(proto::p2p::message::Sum::PexRequest(_)) => Ok(Self::Request),
            None => Err(Report::msg("unable to decode message into PexReceive")),
        }
    }
}

impl From<Pex> for proto::p2p::Message {
    fn from(pex: Pex) -> Self {
        use proto::p2p::{message::Sum, PexAddrs, PexRequest};

        let sum = match pex {
            Pex::Addrs(addrs) => Sum::PexAddrs(PexAddrs { addrs }),
            Pex::Request => Sum::PexRequest(PexRequest {}),
        };

        Self { sum: Some(sum) }
    }
}

pub enum Send {
    Pex(Pex),
}

impl Send {
    pub fn encode<B>(self, buf: &mut B) -> Result<(), proto::Error>
    where
        B: BufMut,
    {
        let msg = match self {
            Self::Pex(pex) => pex,
        };

        msg.encode(buf)
    }
}
