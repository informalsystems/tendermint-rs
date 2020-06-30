use crate::{
    bail,
    errors::{Error, ErrorKind},
    supervisor::Instance,
    types::PeerId,
};

use contracts::{post, pre};
use std::collections::{BTreeSet, HashMap};

/// A mapping from PeerIds to Light Client instances.
/// Keeps track of which peer is deemed the primary peer.
#[derive(Debug)]
pub struct PeerList {
    instances: HashMap<PeerId, Instance>,
    primary: PeerId,
    witnesses: BTreeSet<PeerId>,
    full_nodes: BTreeSet<PeerId>,
    faulty_nodes: BTreeSet<PeerId>,
}

impl PeerList {
    /// Invariant maintained by a `PeerList`
    ///
    /// ## Implements
    /// - [LCD-INV-NODES]
    pub fn invariant(peer_list: &PeerList) -> bool {
        peer_list.full_nodes.is_disjoint(&peer_list.witnesses)
            && peer_list.full_nodes.is_disjoint(&peer_list.faulty_nodes)
            && peer_list.witnesses.is_disjoint(&peer_list.faulty_nodes)
    }

    /// Transition invariant maintained by a `PeerList`
    ///
    /// ## Implements
    /// - [LCD-INV-NODES]
    pub fn transition_invariant(_prev: &PeerList, _next: &PeerList) -> bool {
        true
        // TODO
        // &next.full_nodes | &next.witnesses | &next.faulty_nodes
        //     == &prev.full_nodes | &prev.witnesses | &prev.faulty_nodes
    }

    /// Returns a builder of `PeerList`
    pub fn builder() -> PeerListBuilder {
        PeerListBuilder::default()
    }

    /// Get a reference to the light client instance for the given peer id.
    pub fn get(&self, peer_id: &PeerId) -> Option<&Instance> {
        self.instances.get(peer_id)
    }

    /// Get a mutable reference to the light client instance for the given peer id.
    pub fn get_mut(&mut self, peer_id: &PeerId) -> Option<&mut Instance> {
        self.instances.get_mut(peer_id)
    }

    /// Get a reference to the current primary instance.
    pub fn primary(&self) -> Option<&Instance> {
        self.instances.get(&self.primary)
    }

    /// Get a mutable reference to the current primary instance.
    pub fn primary_mut(&mut self) -> Option<&mut Instance> {
        self.instances.get_mut(&self.primary)
    }

    /// Get a list of references to all the witnesses,
    /// (ie. all peers which are not the primary).
    pub fn witnesses(&self) -> Vec<&Instance> {
        self.instances
            .keys()
            .filter(|peer_id| peer_id != &&self.primary)
            .filter_map(|peer_id| self.get(peer_id))
            .collect()
    }

    /// Remove the given peer from the list of witnesses,
    /// and mark it as faulty. Get a new witness from
    /// the list of full nodes, if there are any left.
    ///
    /// ## Precondition
    /// - The given peer id must not be the primary peer id.
    /// - The given peer must be in the witness list
    #[pre(faulty_witness != self.primary && self.witnesses.contains(&faulty_witness))]
    #[post(Self::invariant(&self))]
    pub fn replace_faulty_witness(&mut self, faulty_witness: PeerId) {
        self.witnesses.remove(&faulty_witness);

        if let Some(new_witness) = self.full_nodes.iter().next().copied() {
            self.witnesses.insert(new_witness);
            self.full_nodes.remove(&new_witness);
        }

        self.faulty_nodes.insert(faulty_witness);
    }

    /// Mark the primary as faulty and swap it for the next available witness, if any.
    ///
    /// ## Errors
    /// - If there are no witness left, returns `ErrorKind::NoWitnessLeft`.
    #[post(ret.is_ok() ==> Self::invariant(&self))]
    pub fn replace_faulty_primary(&mut self) -> Result<(), Error> {
        self.faulty_nodes.insert(self.primary);

        while let Some(peer_id) = self.witnesses.iter().next() {
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
    instances: HashMap<PeerId, Instance>,
    primary: Option<PeerId>,
    witnesses: BTreeSet<PeerId>,
    full_nodes: BTreeSet<PeerId>,
    faulty_nodes: BTreeSet<PeerId>,
}

impl PeerListBuilder {
    /// Register the given peer id and instance as the primary.
    /// Overrides the previous primary if it was already set.
    pub fn primary(mut self, peer_id: PeerId, instance: Instance) -> Self {
        self.primary = Some(peer_id);
        self.instances.insert(peer_id, instance);
        self
    }

    /// Register the given peer id and instance as a witness.
    pub fn witness(mut self, peer_id: PeerId, instance: Instance) -> Self {
        self.instances.insert(peer_id, instance);
        self.witnesses.insert(peer_id);
        self
    }

    /// Register the given peer id and instance as a full node.
    pub fn full_node(mut self, peer_id: PeerId, instance: Instance) -> Self {
        self.instances.insert(peer_id, instance);
        self.full_nodes.insert(peer_id);
        self
    }
    /// Register the given peer id and instance as a faulty node.
    pub fn faulty_node(mut self, peer_id: PeerId, instance: Instance) -> Self {
        self.instances.insert(peer_id, instance);
        self.faulty_nodes.insert(peer_id);
        self
    }

    /// Builds the `PeerList`.
    ///
    /// ## Precondition
    /// - A primary has been set with a call to `PeerListBuilder::primary`.
    #[pre(self.primary.is_some())]
    #[post(PeerList::invariant(&ret))]
    pub fn build(self) -> PeerList {
        PeerList {
            instances: self.instances,
            primary: self.primary.unwrap(),
            witnesses: self.witnesses,
            full_nodes: self.full_nodes,
            faulty_nodes: self.faulty_nodes,
        }
    }
}
