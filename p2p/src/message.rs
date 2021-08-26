use bytes::buf::{Buf, BufMut};
use eyre::Result;
use prost::Message as _;

use tendermint_proto as proto;

use crate::transport::StreamId;

pub enum Receive {
    Pex(PexReceive),
}

impl From<proto::p2p::message::Sum> for Receive {
    fn from(raw: proto::p2p::message::Sum) -> Self {
        match raw {
            proto::p2p::message::Sum::PexAddrs(proto::p2p::PexAddrs { addrs }) => {
                Self::Pex(PexReceive::Addrs(addrs))
            }
            proto::p2p::message::Sum::PexRequest(_) => Self::Pex(PexReceive::Request),
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

pub enum PexReceive {
    Addrs(Vec<proto::p2p::NetAddress>),
    Request,
}

pub enum Send {
    Pex(PexSend),
}

impl Send {
    pub fn encode<B>(self, buf: &mut B) -> Result<()>
    where
        B: BufMut,
    {
        let msg = match self {
            Self::Pex(pex) => proto::p2p::message::Sum::from(pex),
        };

        Ok(msg.encode(buf))
    }
}

pub trait Outgoing: std::marker::Send + Sync {}

pub enum PexSend {
    Addrs(Vec<proto::p2p::NetAddress>),
    Request,
}

impl From<PexSend> for proto::p2p::message::Sum {
    fn from(msg: PexSend) -> Self {
        match msg {
            PexSend::Addrs(addrs) => {
                proto::p2p::message::Sum::PexAddrs(proto::p2p::PexAddrs { addrs })
            }
            PexSend::Request => proto::p2p::message::Sum::PexRequest(proto::p2p::PexRequest {}),
        }
    }
}

impl Outgoing for PexSend {}
