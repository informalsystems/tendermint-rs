//! Provides a peer list for use within the `Supervisor`

extern crate stainless;
use stainless::*;

#[derive(Clone)]
pub struct ListSet<T> {
    list: List<T>,
}

#[derive(Clone)]
pub struct ListMap<K, V> {
    list: List<(K, V)>,
}

#[derive(Clone)]
enum List<T> {
    Nil,
    Cons(T, Box<List<T>>),
}

impl ListSet<u128> {
    pub fn empty() -> Self {
        ListSet { list: List::Nil }
    }

    pub fn is_disjoint(&self, other: &ListSet<u128>) -> bool {
        is_equal(
            &self.list.contents().intersection(other.list.contents()),
            &Set::empty(),
        )
    }

    pub fn contains(&self, t: &u128) -> bool {
        self.list.contents().contains(&t)
    }

    pub fn remove(self, t: &u128) -> Self {
        Self {
            list: self.list.remove(t),
        }
    }

    pub fn add(self, t: u128) -> Self {
        Self {
            list: self.list.add(t),
        }
    }

    pub fn first(&self) -> Option<u128> {
        match &self.list {
            List::Cons(t, _) => Some(t.clone()),
            _ => None,
        }
    }
}

impl<V> ListMap<u128, V> {
    pub fn get(&self, key: &u128) -> Option<&V> {
        self.list.get(key)
    }

    pub fn contains(&self, key: &u128) -> bool {
        self.list.key_set().contains(&key)
    }

    pub fn contains_all(&self, keys: &ListSet<u128>) -> bool {
        is_equal(
            &self.list.key_set().intersection(keys.list.contents()),
            &keys.list.contents(),
        )
    }
}

fn is_equal<'a>(s1: &Set<&'a u128>, s2: &Set<&'a u128>) -> bool {
    s1.is_subset_of(s2) && s2.is_subset_of(s1)
}

impl List<u128> {
    #[measure(self)]
    pub fn contents(&self) -> Set<&u128> {
        match self {
            List::Nil => Set::empty(),
            List::Cons(head, tail) => tail.contents().add(head),
        }
    }

    pub fn remove(self, t: &u128) -> Self {
        match self {
            List::Nil => self,
            List::Cons(head, tail) if head == *t => *tail,
            List::Cons(head, tail) => List::Cons(head, Box::new(tail.remove(t))),
        }
    }

    pub fn add(self, t: u128) -> Self {
        match self {
            List::Nil => List::Cons(t, Box::new(List::Nil)),
            _ => List::Cons(t, Box::new(self)),
        }
    }
}

impl<V> List<(u128, V)> {
    pub fn key_set(&self) -> Set<&u128> {
        match self {
            List::Nil => Set::empty(),
            List::Cons(head, tail) => tail.key_set().add(&head.0),
        }
    }

    pub fn get(&self, key: &u128) -> Option<&V> {
        match &self {
            List::Nil => None,
            List::Cons(head, _) if head.0 == *key => Some(&head.1),
            List::Cons(_, tail) => tail.get(key),
        }
    }
}

// Copied imports from the `light-client` crate:
macro_rules! bail {
    ($kind:expr) => {
        return Err(Box::new($kind));
    };
}

/// Node IDs
// u128 was replaced by a simple u128 to make hashing easier.

pub enum ErrorKind {
    NoWitnessLeft { context: Option<Box<ErrorKind>> },
}

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
    #[pre(&faulty_witness != &self.primary && self.witnesses.contains(&faulty_witness))]
    #[post(Self::invariant(&ret.0))]
    pub fn replace_faulty_witness(mut self, faulty_witness: u128) -> (Self, Option<u128>) {
        let mut result = None;

        self.witnesses = self.witnesses.remove(&faulty_witness);

        if let Some(new_witness) = self.full_nodes.first() {
            self.witnesses = self.witnesses.add(new_witness);
            self.full_nodes = self.full_nodes.remove(&new_witness);
            result = Some(new_witness);
        }

        self.faulty_nodes = self.faulty_nodes.add(faulty_witness);

        (self, result)
    }

    /// Mark the primary as faulty and swap it for the next available witness, if any.
    /// Returns the new primary on success.
    ///
    /// ## Errors
    /// - If there are no witness left, returns `ErrorKind::NoWitnessLeft`.
    #[post((matches!(ret, Ok(_))).implies(Self::invariant(&self)))]
    pub fn replace_faulty_primary(
        mut self,
        primary_error: Option<Box<ErrorKind>>,
    ) -> Result<(Self, u128), Box<ErrorKind>> {
        self.faulty_nodes = self.faulty_nodes.add(self.primary);

        if let Some(new_primary) = self.witnesses.first() {
            self.primary = new_primary;
            self.witnesses = self.witnesses.remove(&new_primary);
            Ok((self, new_primary))
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
