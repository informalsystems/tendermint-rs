use super::{Chain, Id, Registry};
use std::sync::RwLockReadGuard;

/// Wrapper for a `RwLockReadGuard<'static, Registry>`, allowing access to
/// global information about particular Tendermint networks / "chains"
pub struct Guard<'lock>(RwLockReadGuard<'lock, Registry>);

impl<'lock> From<RwLockReadGuard<'lock, Registry>> for Guard<'lock> {
    fn from(guard: RwLockReadGuard<'lock, Registry>) -> Guard<'lock> {
        Guard(guard)
    }
}

impl<'lock> Guard<'lock> {
    /// Get information about a particular chain ID (if registered)
    pub fn get_chain(&self, chain_id: &Id) -> Option<&Chain> {
        self.0.get_chain(chain_id)
    }
}
