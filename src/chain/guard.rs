use super::{Chain, Id};
use std::{collections::BTreeMap, sync::RwLockReadGuard};

/// Wrapper for a `RwLockReadGuard<'static, BTreeMap<Id, Chain>>`, allowing access to
/// global information about particular Tendermint networks / "chains"
pub struct Guard<'lock>(RwLockReadGuard<'lock, BTreeMap<Id, Chain>>);

impl<'lock> From<RwLockReadGuard<'lock, BTreeMap<Id, Chain>>> for Guard<'lock> {
    fn from(guard: RwLockReadGuard<'lock, BTreeMap<Id, Chain>>) -> Guard<'lock> {
        Guard(guard)
    }
}

impl<'lock> Guard<'lock> {
    /// Get information about a particular chain ID (if registered)
    pub fn chain(&self, chain_id: Id) -> Option<&Chain> {
        self.0.get(&chain_id)
    }
}
