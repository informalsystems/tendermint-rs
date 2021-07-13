//! Provides a peer list for use within the `Supervisor`

extern crate stainless;
use stainless::*;

mod list;
use list::*;

// Copied imports from the `light-client` crate:
macro_rules! bail {
    ($kind:expr) => {
        return Err(Box::new($kind));
    };
}

/// Node IDs
// PeerId was replaced by a simple u128 to make hashing easier.

pub enum ErrorKind {
    NoWitnessLeft { context: Option<Box<ErrorKind>> },
}

pub struct Tuple<T, S>(T, S);

// Copied imports end.

/// A generic container mapping `u128`s to some type `T`,
/// which keeps track of the primary peer, witnesses, full nodes,
/// and faulty nodes. Provides lifecycle methods to swap the primary,
/// mark witnesses as faulty, and maintains an `invariant` for
/// correctness.
#[derive(Clone)]
pub struct PeerList<T> {
    values: ListMap<u128, T>,
    primary: u128,
    witnesses: ListSet<u128>,
    full_nodes: ListSet<u128>,
    faulty_nodes: ListSet<u128>,
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
            && peer_list.values.contains(&peer_list.primary)
            && peer_list.values.contains_all(&peer_list.witnesses)
            && peer_list.values.contains_all(&peer_list.full_nodes)
            && peer_list.values.contains_all(&peer_list.faulty_nodes)
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
    pub fn get(&self, peer_id: &u128) -> Option<&T> {
        self.values.get(peer_id)
    }

    /// Get a mutable reference to the light client instance for the given peer id.
    // pub fn get_mut(&mut self, peer_id: &u128) -> Option<&mut T> {
    //     self.values.get_mut(peer_id)
    // }

    /// Get current primary peer id.
    pub fn primary_id(&self) -> u128 {
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
    pub fn witnesses_ids(&self) -> &ListSet<u128> {
        &self.witnesses
    }

    /// Get all the full nodes peer ids
    pub fn full_nodes_ids(&self) -> &ListSet<u128> {
        &self.full_nodes
    }

    /// Get all the faulty nodes peer ids
    pub fn faulty_nodes_ids(&self) -> &ListSet<u128> {
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
    #[pre(
        Self::invariant(&self)
        && !(faulty_witness == self.primary)
        && self.witnesses.contains(&faulty_witness)
    )]
    #[post(
        Self::invariant(&ret.0)
        && !ret.0.witnesses.contains(&faulty_witness)
        && ret.0.faulty_nodes.contains(&faulty_witness)
    )]
    pub fn replace_faulty_witness(self, faulty_witness: u128) -> Tuple<Self, Option<u128>> {
        let mut result = None;

        let mut new_full = self.full_nodes;
        let mut new_witnesses = self.witnesses.remove(&faulty_witness);

        if let Some(new_witness) = new_full.first() {
            new_witnesses = new_witnesses.insert(new_witness);
            new_full = new_full.remove(&new_witness);
            result = Some(new_witness);
        }

        let new_faulty = self.faulty_nodes.insert(faulty_witness);
        Tuple(
            Self {
                full_nodes: new_full,
                witnesses: new_witnesses,
                faulty_nodes: new_faulty,
                ..self
            },
            result,
        )
    }

    /// Mark the primary as faulty and swap it for the next available witness, if any.
    /// Returns the new primary on success.
    ///
    /// ## Errors
    /// - If there are no witness left, returns `ErrorKind::NoWitnessLeft`.
    #[pre(Self::invariant(&self))]
    #[post((matches!(ret, Ok(_))).implies(
               match ret {
                   Ok(Tuple(new_list, _)) => Self::invariant(&new_list)
                       && self.primary != new_list.primary
                       && new_list.faulty_nodes.contains(&self.primary)
                       && self.witnesses.contains(&new_list.primary),
                   _ => false,
               }
           ))]
    pub fn replace_faulty_primary(
        self,
        primary_error: Option<Box<ErrorKind>>,
    ) -> Result<Tuple<Self, u128>, Box<ErrorKind>> {
        let new_faulty = self.faulty_nodes.insert(self.primary);

        if let Some(new_primary) = self.witnesses.first() {
            let new_witnesses = self.witnesses.remove(&new_primary);

            let ret = Self {
                primary: new_primary,
                faulty_nodes: new_faulty,
                witnesses: new_witnesses,
                ..self
            };
            Ok(Tuple(ret, new_primary))
        } else if let Some(err) = primary_error {
            bail!(ErrorKind::NoWitnessLeft { context: Some(err) })
        } else {
            bail!(ErrorKind::NoWitnessLeft { context: None })
        }
    }

    /// Get a reference to the underlying `HashMap`
    pub fn values(&self) -> &ListMap<u128, T> {
        &self.values
    }
    /// Consume into the underlying `HashMap`
    pub fn into_values(self) -> ListMap<u128, T> {
        self.values
    }
}
