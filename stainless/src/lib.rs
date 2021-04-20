//! Provides a peer list for use within the `Supervisor`

extern crate stainless;

use stainless::*;
use std::collections::{BTreeSet, HashMap};

// Copied imports from the `light-client` crate:
macro_rules! bail {
    ($kind:expr) => {
        return Err(Box::new($kind));
    };
}

pub const LENGTH: usize = 20;

/// Node IDs
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PeerId([u8; LENGTH]);

type Error = Box<ErrorKind>;
pub enum ErrorKind {
    NoWitnessLeft { context: Option<Error> },
}

// Copied imports end.

/// A generic container mapping `PeerId`s to some type `T`,
/// which keeps track of the primary peer, witnesses, full nodes,
/// and faulty nodes. Provides lifecycle methods to swap the primary,
/// mark witnesses as faulty, and maintains an `invariant` for
/// correctness.
#[derive(Clone)]
pub struct PeerList<T> {
    values: HashMap<PeerId, T>,
    primary: PeerId,
    witnesses: BTreeSet<PeerId>,
    full_nodes: BTreeSet<PeerId>,
    faulty_nodes: BTreeSet<PeerId>,
}

impl<T> PeerList<T> {
    /// Invariant maintained by a `PeerList`
    ///
    /// ## Implements
    /// - [LCD-INV-NODES]
    pub fn invariant(peer_list: &PeerList<T>) -> bool {
        peer_list.full_nodes.is_disjoint(&peer_list.witnesses)
            && peer_list.full_nodes.is_disjoint(&peer_list.faulty_nodes)
            && peer_list.witnesses.is_disjoint(&peer_list.faulty_nodes)
            && !peer_list.witnesses.contains(&peer_list.primary)
            && !peer_list.full_nodes.contains(&peer_list.primary)
            && !peer_list.faulty_nodes.contains(&peer_list.primary)
            && peer_list.values.contains_key(&peer_list.primary)
            && peer_list
                .witnesses
                .iter()
                .all(|id| peer_list.values.contains_key(id))
            && peer_list
                .full_nodes
                .iter()
                .all(|id| peer_list.values.contains_key(id))
            && peer_list
                .faulty_nodes
                .iter()
                .all(|id| peer_list.values.contains_key(id))
    }

    /// Transition invariant maintained by a `PeerList`
    ///
    /// ## Implements
    /// - [LCD-INV-NODES]
    pub fn transition_invariant(_prev: &PeerList<T>, _next: &PeerList<T>) -> bool {
        true
        // TODO: Implement transition invariant
        // &next.full_nodes | &next.witnesses | &next.faulty_nodes
        //     == &prev.full_nodes | &prev.witnesses | &prev.faulty_nodes
    }

    /// Get a reference to the light client instance for the given peer id.
    pub fn get(&self, peer_id: &PeerId) -> Option<&T> {
        self.values.get(peer_id)
    }

    /// Get a mutable reference to the light client instance for the given peer id.
    // pub fn get_mut(&mut self, peer_id: &PeerId) -> Option<&mut T> {
    //     self.values.get_mut(peer_id)
    // }

    /// Get current primary peer id.
    pub fn primary_id(&self) -> PeerId {
        self.primary
    }

    /// Get a reference to the current primary instance.
    pub fn primary(&self) -> &T {
        self.values.get(&self.primary).unwrap() // SAFETY: Enforced by invariant
    }

    /// Get a mutable reference to the current primary instance.
    // pub fn primary_mut(&mut self) -> &mut T {
    //     self.values.get_mut(&self.primary).unwrap() // SAFETY: Enforced by invariant
    // }

    /// Get all the witnesses peer ids
    pub fn witnesses_ids(&self) -> &BTreeSet<PeerId> {
        &self.witnesses
    }

    /// Get all the full nodes peer ids
    pub fn full_nodes_ids(&self) -> &BTreeSet<PeerId> {
        &self.full_nodes
    }

    /// Get all the faulty nodes peer ids
    pub fn faulty_nodes_ids(&self) -> &BTreeSet<PeerId> {
        &self.faulty_nodes
    }

    /// Remove the given peer from the list of witnesses,
    /// and mark it as faulty. Get a new witness from
    /// the list of full nodes, if there are any left.
    /// Returns the new witness, if any.
    ///
    /// ## Precondition
    /// - The given peer id must not be the primary peer id.
    /// - The given peer must be in the witness list
    #[pre(faulty_witness != self.primary && self.witnesses.contains(&faulty_witness))]
    #[post(Self::invariant(&self))]
    pub fn replace_faulty_witness(&mut self, faulty_witness: PeerId) -> Option<PeerId> {
        let mut result = None;

        self.witnesses.remove(&faulty_witness);

        if let Some(new_witness) = self.full_nodes.iter().next().copied() {
            self.witnesses.insert(new_witness);
            self.full_nodes.remove(&new_witness);
            result = Some(new_witness);
        }

        self.faulty_nodes.insert(faulty_witness);

        result
    }

    /// Mark the primary as faulty and swap it for the next available witness, if any.
    /// Returns the new primary on success.
    ///
    /// ## Errors
    /// - If there are no witness left, returns `ErrorKind::NoWitnessLeft`.
    #[post(ret.is_ok().implies(Self::invariant(&self)))]
    pub fn replace_faulty_primary(
        &mut self,
        primary_error: Option<Error>,
    ) -> Result<PeerId, Error> {
        self.faulty_nodes.insert(self.primary);

        if let Some(new_primary) = self.witnesses.iter().next().copied() {
            self.primary = new_primary;
            self.witnesses.remove(&new_primary);
            Ok(new_primary)
        } else if let Some(err) = primary_error {
            bail!(ErrorKind::NoWitnessLeft { context: Some(err) })
        } else {
            bail!(ErrorKind::NoWitnessLeft { context: None })
        }
    }

    /// Get a reference to the underlying `HashMap`
    pub fn values(&self) -> &HashMap<PeerId, T> {
        &self.values
    }
    /// Consume into the underlying `HashMap`
    pub fn into_values(self) -> HashMap<PeerId, T> {
        self.values
    }
}
