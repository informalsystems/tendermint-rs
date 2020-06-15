use crate::{
    bail,
    errors::{Error, ErrorKind},
    supervisor::Instance,
    types::PeerId,
};

use contracts::pre;
use std::collections::HashMap;

/// A mapping from PeerIds to Light Client instances.
/// Keeps track of which peer is deemed the primary peer.
#[derive(Debug)]
pub struct PeerList {
    peers: HashMap<PeerId, Instance>,
    primary: PeerId,
}

impl PeerList {
    /// Returns a builder of `PeerList`
    pub fn builder() -> PeerListBuilder {
        PeerListBuilder::default()
    }

    /// Get a reference to the light client instance for the given peer id.
    pub fn get(&self, peer_id: &PeerId) -> Option<&Instance> {
        self.peers.get(peer_id)
    }

    /// Get a mutable reference to the light client instance for the given peer id.
    pub fn get_mut(&mut self, peer_id: &PeerId) -> Option<&mut Instance> {
        self.peers.get_mut(peer_id)
    }

    /// Get a reference to the current primary instance.
    pub fn primary(&self) -> Option<&Instance> {
        self.peers.get(&self.primary)
    }

    /// Get a mutable reference to the current primary instance.
    pub fn primary_mut(&mut self) -> Option<&mut Instance> {
        self.peers.get_mut(&self.primary)
    }

    /// Get a list of references to all the witnesses,
    /// (ie. all peers which are not the primary).
    pub fn witnesses(&self) -> Vec<&Instance> {
        self.peers
            .keys()
            .filter(|peer_id| peer_id != &&self.primary)
            .filter_map(|peer_id| self.get(peer_id))
            .collect()
    }

    /// Remove the given peer from the list of witnesses.
    ///
    /// ## Precondition
    /// - The given peer id must not be the primary peer id.
    #[pre(peer_id != &self.primary)]
    pub fn remove_witness(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);
    }

    /// Swap the primary for the next available witness, if any.
    ///
    /// ## Errors
    /// - If there are no witness left, returns `ErrorKind::NoWitnessLeft`.
    pub fn swap_primary(&mut self) -> Result<(), Error> {
        while let Some(peer_id) = self.peers.keys().next() {
            if peer_id != &self.primary {
                self.primary = *peer_id;
                return Ok(());
            }
        }

        bail!(ErrorKind::NoWitnessLeft)
    }
}

/// A builder of `PeerList` with a fluent API.
#[derive(Default)]
pub struct PeerListBuilder {
    primary: Option<PeerId>,
    peers: HashMap<PeerId, Instance>,
}

impl PeerListBuilder {
    /// Register the given peer id and instance as the primary.
    /// Overrides the previous primary if it was already set.
    pub fn primary(mut self, peer_id: PeerId, instance: Instance) -> Self {
        self.primary = Some(peer_id);
        self.peers.insert(peer_id, instance);
        self
    }

    /// Register the given peer id and instance as a witness.
    pub fn witness(mut self, peer_id: PeerId, instance: Instance) -> Self {
        self.peers.insert(peer_id, instance);
        self
    }

    /// Builds the `PeerList`.
    ///
    /// ## Precondition
    /// - A primary has been set with a call to `PeerListBuilder::primary`.
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
