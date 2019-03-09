//! Registry of information about known Tendermint blockchain networks

use super::{Chain, Guard, Id};
use crate::error::{KmsError, KmsErrorKind::ConfigError};
use std::{collections::BTreeMap, sync::RwLock};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::default();
}

/// Registry of blockchain networks known to the KMS
// The `RwLock` is a bit of futureproofing as this data structure is for the
// most part "immutable". New chains should be registered at boot time.
// The only case in which this structure may change is in the event of
// runtime configuration reloading, so the `RwLock` is included as
// futureproofing for such a feature.
// See: <https://github.com/tendermint/kms/issues/183>
#[derive(Default)]
pub struct Registry(RwLock<BTreeMap<Id, Chain>>);

impl Registry {
    /// Acquire a read-only (concurrent) lock to the internal chain registry
    pub fn get(&self) -> Guard {
        // TODO(tarcieri): better handle `PoisonError` here?
        self.0.read().unwrap().into()
    }

    /// Register a chain with the registry
    pub fn register(&self, chain: Chain) -> Result<(), KmsError> {
        // TODO(tarcieri): better handle `PoisonError` here?
        let mut chain_map = self.0.write().unwrap();

        let chain_id = chain.id;

        if chain_map.insert(chain_id, chain).is_none() {
            Ok(())
        } else {
            // TODO(tarcieri): handle updating the set of registered chains
            fail!(ConfigError, "chain ID already registered: {}", chain_id);
        }
    }
}
