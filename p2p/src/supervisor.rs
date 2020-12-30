use std::collections::HashMap;

use eyre::Result;

use crate::message;
use crate::peer::{self, Peer, PeerId};
use crate::transport::Transport;

pub enum Output {
    Peer(PeerId, message::Receive),
}

struct Supervisor<S, T>
where
    S: peer::State,
    T: Transport,
{
    peers: HashMap<PeerId, Peer<S>>,
    transport: T,
}

impl<S, T> Supervisor<S, T>
where
    S: peer::State,
    T: Transport,
{
    pub fn recv(&self) -> Result<Output> {
        todo!()
    }

    pub fn send(&self, peer_id: PeerId, msg: message::Send) -> Result<()> {
        todo!()
    }
}
