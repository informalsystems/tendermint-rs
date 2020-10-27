//! Provides a peer list for use within the `Supervisor`

use crate::{
    bail,
    errors::{Error, ErrorKind},
};

use contracts::{post, pre};
use std::{collections::HashMap, collections::HashSet, hash::Hash};

/// A generic container mapping peers of type `K` to values of type `V`,
/// which keeps track of the primary peer, witnesses, full nodes,
/// and faulty nodes. Provides lifecycle methods to swap the primary,
/// mark witnesses as faulty, and maintains an `invariant` for
/// correctness.
#[derive(Clone, Debug)]
pub struct PeerList<K, V> {
    values: HashMap<K, V>,
    primary: K,
    witnesses: HashSet<K>,
    full_nodes: HashSet<K>,
    faulty_nodes: HashSet<K>,
}

impl<K, V> PeerList<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Invariant maintained by a `PeerList`
    ///
    /// ## Implements
    /// - [LCD-INV-NODES]
    pub fn invariant(peer_list: &PeerList<K, V>) -> bool {
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
    pub fn transition_invariant(_prev: &PeerList<K, V>, _next: &PeerList<K, V>) -> bool {
        true
        // TODO: Implement transition invariant
        // &next.full_nodes | &next.witnesses | &next.faulty_nodes
        //     == &prev.full_nodes | &prev.witnesses | &prev.faulty_nodes
    }

    /// Returns a builder of `PeerList`
    pub fn builder() -> PeerListBuilder<K, V> {
        PeerListBuilder::default()
    }

    /// Get a reference to the light client instance for the given peer id.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.values.get(key)
    }

    /// Get a mutable reference to the light client instance for the given peer id.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.values.get_mut(key)
    }

    /// Get current primary peer id.
    pub fn primary_id(&self) -> &K {
        &self.primary
    }

    /// Get a reference to the current primary instance.
    pub fn primary(&self) -> &V {
        self.values.get(&self.primary).unwrap() // SAFETY: Enforced by invariant
    }

    /// Get a mutable reference to the current primary instance.
    pub fn primary_mut(&mut self) -> &mut V {
        self.values.get_mut(&self.primary).unwrap() // SAFETY: Enforced by invariant
    }

    /// Get all the witnesses peer ids
    pub fn witnesses_ids(&self) -> &HashSet<K> {
        &self.witnesses
    }

    /// Get all the full nodes peer ids
    pub fn full_nodes_ids(&self) -> &HashSet<K> {
        &self.full_nodes
    }

    /// Get all the faulty nodes peer ids
    pub fn faulty_nodes_ids(&self) -> &HashSet<K> {
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
    pub fn replace_faulty_witness(&mut self, faulty_witness: K) -> Option<K> {
        let mut result = None;

        self.witnesses.remove(&faulty_witness);

        if let Some(new_witness) = self.full_nodes.iter().next().cloned() {
            self.witnesses.insert(new_witness.clone());
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
    #[post(ret.is_ok() ==> Self::invariant(&self))]
    pub fn replace_faulty_primary(&mut self, primary_error: Option<Error>) -> Result<K, Error> {
        self.faulty_nodes.insert(self.primary.clone());

        if let Some(new_primary) = self.witnesses.iter().next().cloned() {
            self.primary = new_primary.clone();
            self.witnesses.remove(&new_primary);
            Ok(new_primary)
        } else if let Some(err) = primary_error {
            bail!(ErrorKind::NoWitnessLeft.context(err))
        } else {
            bail!(ErrorKind::NoWitnessLeft)
        }
    }

    /// Get a reference to the underlying `HashMap`
    pub fn values(&self) -> &HashMap<K, V> {
        &self.values
    }
    /// Consume into the underlying `HashMap`
    pub fn into_values(self) -> HashMap<K, V> {
        self.values
    }
}

/// A builder of `PeerList` with a fluent API.
#[must_use]
pub struct PeerListBuilder<K, V> {
    values: HashMap<K, V>,
    primary: Option<K>,
    witnesses: HashSet<K>,
    full_nodes: HashSet<K>,
    faulty_nodes: HashSet<K>,
}

// This instance must be derived manually because the automatically
// derived instance constrains K and V to be Default.
// See https://github.com/rust-lang/rust/issues/26925
impl<K, V> Default for PeerListBuilder<K, V>
where
    K: Clone + Eq + Hash,
{
    fn default() -> Self {
        Self {
            values: Default::default(),
            primary: Default::default(),
            witnesses: Default::default(),
            full_nodes: Default::default(),
            faulty_nodes: Default::default(),
        }
    }
}

impl<K, V> PeerListBuilder<K, V>
where
    K: Clone + Eq + Hash,
{
    /// Register the given key and instance as the primary.
    /// Overrides the previous primary if it was already set.
    pub fn primary(&mut self, key: K, value: V) {
        self.primary = Some(key.clone());
        self.values.insert(key, value);
    }

    /// Register the given key and value as a witness.
    #[pre(self.primary.as_ref() != Some(&key))]
    pub fn witness(&mut self, key: K, value: V) {
        self.values.insert(key.clone(), value);
        self.witnesses.insert(key);
    }

    /// Register the given key and value as a full node.
    #[pre(self.primary.as_ref() != Some(&key))]
    pub fn full_node(&mut self, key: K, value: V) {
        self.values.insert(key.clone(), value);
        self.full_nodes.insert(key);
    }

    /// Register the given key and value as a faulty node.
    #[pre(self.primary.as_ref() != Some(&key))]
    pub fn faulty_node(&mut self, key: K, value: V) {
        self.values.insert(key.clone(), value);
        self.faulty_nodes.insert(key);
    }

    /// Builds the `PeerList`.
    ///
    /// ## Precondition
    /// - A primary has been set with a call to `PeerListBuilder::primary`.
    #[pre(self.primary.is_some())]
    #[post(PeerList::invariant(&ret))]
    pub fn build(self) -> PeerList<K, V> {
        PeerList {
            values: self.values,
            primary: self.primary.unwrap(),
            witnesses: self.witnesses,
            full_nodes: self.full_nodes,
            faulty_nodes: self.faulty_nodes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PeerId;

    trait ToVec<T> {
        fn to_vec(&self) -> Vec<T>;
    }

    impl ToVec<PeerId> for HashSet<PeerId> {
        fn to_vec(&self) -> Vec<PeerId> {
            self.iter().copied().collect()
        }
    }

    fn a() -> PeerId {
        "6de6deefcc12585340af922a0dd332084546a207".parse().unwrap()
    }
    fn b() -> PeerId {
        "17a7e0367b3bcf7323d96217b51c5fe5b096a7b5".parse().unwrap()
    }
    fn c() -> PeerId {
        "2a515002827b5cc0c6fdb73bcb162f516fad75c8".parse().unwrap()
    }
    fn d() -> PeerId {
        "da918eef62d986812b4e6271de78db4ec52594eb".parse().unwrap()
    }
    fn dummy_peer_list() -> PeerList<PeerId, u32> {
        let mut builder = PeerList::builder();
        builder.primary(a(), 1_u32);
        builder.witness(b(), 2_u32);
        builder.full_node(c(), 3_u32);
        builder.build()
    }

    #[test]
    fn builder_succeeds() {
        let peer_list = dummy_peer_list();

        assert!(PeerList::invariant(&peer_list));
        assert_eq!(peer_list.primary(), &1);
        assert_eq!(peer_list.primary_id(), &a());
        assert_eq!(peer_list.witnesses_ids().to_vec(), vec![b()]);
        assert_eq!(peer_list.full_nodes_ids().to_vec(), vec![c()]);
        assert!(peer_list.faulty_nodes_ids().is_empty());
    }

    #[test]
    #[should_panic(expected = "Pre-condition of build violated")]
    fn builder_fails_if_no_primary() {
        let mut builder = PeerList::builder();
        builder.witness(b(), 2_u32);
        builder.full_node(c(), 3_u32);
        let _ = builder.build();
        unreachable!();
    }

    #[test]
    fn replace_faulty_primary_succeeds() {
        let mut peer_list = dummy_peer_list();
        assert_eq!(peer_list.primary(), &1);
        let new_primary = peer_list.replace_faulty_primary(None);
        assert_eq!(new_primary.unwrap(), b());
        assert_eq!(peer_list.primary(), &2);
        assert!(peer_list.witnesses_ids().is_empty());
    }

    #[test]
    fn replace_faulty_primary_fails_if_no_more_witnesses() {
        let mut peer_list = dummy_peer_list();
        let _ = peer_list.replace_faulty_primary(None).unwrap();
        let new_primary = peer_list.replace_faulty_primary(None);
        assert_eq!(
            new_primary.err().map(|e| e.kind().clone()),
            Some(ErrorKind::NoWitnessLeft)
        );
    }

    #[test]
    fn replace_faulty_witness_succeeds() {
        let mut peer_list = dummy_peer_list();
        assert_eq!(peer_list.primary(), &1);
        assert_eq!(peer_list.witnesses_ids().to_vec(), vec![b()]);
        let new_witness = peer_list.replace_faulty_witness(b());
        assert_eq!(new_witness, Some(c()));
        assert_eq!(peer_list.primary(), &1);
        assert_eq!(peer_list.witnesses_ids().to_vec(), vec![c()]);
        assert!(peer_list.full_nodes_ids().is_empty());
    }

    #[test]
    #[should_panic(expected = "Pre-condition of replace_faulty_witness violated")]
    fn replace_faulty_witness_fails_if_not_witness() {
        let mut peer_list = dummy_peer_list();
        let _ = peer_list.replace_faulty_witness(d());
        unreachable!();
    }
}
