//! Registry of information about known Tendermint blockchain networks

use super::{Chain, Guard, Id};
use crate::{
    error::{KmsError, KmsErrorKind::*},
    keyring,
};
use std::{collections::BTreeMap, sync::RwLock};

lazy_static! {
    pub static ref REGISTRY: GlobalRegistry = GlobalRegistry::default();
}

/// Registry of blockchain networks known to the KMS
#[derive(Default)]
pub struct Registry(BTreeMap<Id, Chain>);

impl Registry {
    /// Add a key to a keyring for a chain stored in the registry
    pub fn add_to_keyring(
        &mut self,
        chain_id: &Id,
        signer: keyring::ed25519::Signer,
    ) -> Result<(), KmsError> {
        // TODO(tarcieri):
        let chain = self.0.get_mut(chain_id).ok_or_else(|| {
            err!(
                InvalidKey,
                "can't add signer {} to unregistered chain: {}",
                signer.provider(),
                chain_id
            )
        })?;

        chain.keyring.add(signer)
    }

    /// Register a `Chain` with the registry
    pub fn register_chain(&mut self, chain: Chain) -> Result<(), KmsError> {
        let chain_id = chain.id;

        if self.0.insert(chain_id, chain).is_none() {
            Ok(())
        } else {
            // TODO(tarcieri): handle updating the set of registered chains
            fail!(ConfigError, "chain ID already registered: {}", chain_id);
        }
    }

    /// Get information about a particular chain ID (if registered)
    pub fn get_chain(&self, chain_id: &Id) -> Option<&Chain> {
        self.0.get(chain_id)
    }
}

/// Global registry of blockchain networks known to the KMS
// NOTE: The `RwLock` is a bit of futureproofing as this data structure is for the
// most part "immutable". New chains should be registered at boot time.
// The only case in which this structure may change is in the event of
// runtime configuration reloading, so the `RwLock` is included as
// futureproofing for such a feature.
//
// See: <https://github.com/tendermint/kms/issues/183>
#[derive(Default)]
pub struct GlobalRegistry(pub(super) RwLock<Registry>);

impl GlobalRegistry {
    /// Acquire a read-only (concurrent) lock to the internal chain registry
    pub fn get(&self) -> Guard {
        // TODO(tarcieri): better handle `PoisonError` here?
        self.0.read().unwrap().into()
    }

    /// Register a chain with the registry
    pub fn register(&self, chain: Chain) -> Result<(), KmsError> {
        // TODO(tarcieri): better handle `PoisonError` here?
        let mut registry = self.0.write().unwrap();
        registry.register_chain(chain)
    }
}
