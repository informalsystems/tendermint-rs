use crate::{
    bail,
    errors::{Error, ErrorKind},
    supervisor::Instance,
    types::PeerId,
};

use contracts::pre;
use std::collections::HashMap;

#[derive(Debug)]
pub struct PeerList {
    peers: HashMap<PeerId, Instance>,
    primary: PeerId,
}

impl PeerList {
    pub fn builder() -> PeerListBuilder {
        PeerListBuilder::default()
    }

    pub fn get(&self, peer_id: &PeerId) -> Option<&Instance> {
        self.peers.get(peer_id)
    }

    pub fn get_mut(&mut self, peer_id: &PeerId) -> Option<&mut Instance> {
        self.peers.get_mut(peer_id)
    }

    pub fn primary(&self) -> Option<&Instance> {
        self.peers.get(&self.primary)
    }

    pub fn primary_mut(&mut self) -> Option<&mut Instance> {
        self.peers.get_mut(&self.primary)
    }

    pub fn secondaries(&self) -> Vec<&Instance> {
        self.peers
            .keys()
            .filter(|peer_id| peer_id != &&self.primary)
            .filter_map(|peer_id| self.get(peer_id))
            .collect()
    }

    #[pre(peer_id != &self.primary)]
    pub fn remove_secondary(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);
    }

    pub fn swap_primary(&mut self) -> Result<(), Error> {
        if let Some(peer_id) = self.peers.keys().next() {
            if peer_id != &self.primary {
                self.primary = *peer_id;
                return Ok(());
            }
        }

        bail!(ErrorKind::NoValidPeerLeft)
    }
}

#[derive(Default)]
pub struct PeerListBuilder {
    primary: Option<PeerId>,
    peers: HashMap<PeerId, Instance>,
}

impl PeerListBuilder {
    pub fn primary(mut self, primary: PeerId) -> Self {
        self.primary = Some(primary);
        self
    }

    pub fn peer(mut self, peer_id: PeerId, instance: Instance) -> Self {
        self.peers.insert(peer_id, instance);
        self
    }

    #[pre(
        self.primary.is_some() && self.peers.contains_key(self.primary.as_ref().unwrap())
    )]
    pub fn build(self) -> PeerList {
        PeerList {
            primary: self.primary.unwrap(),
            peers: self.peers,
        }
    }
}
