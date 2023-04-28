//! Fork evidence data structures and interfaces.

use contracts::contract_trait;
pub use tendermint::evidence::Evidence;
use tendermint::Hash;

use crate::{components::io::IoError, verifier::types::PeerId};

/// Interface for reporting evidence to full nodes, typically via the RPC client.
#[contract_trait]
#[allow(missing_docs)] // This is required because of the `contracts` crate (TODO: open/link issue)
pub trait EvidenceReporter: Send + Sync {
    /// Report evidence to all connected full nodes.
    fn report(&self, e: Evidence, peer: PeerId) -> Result<Hash, IoError>;
}

#[cfg(feature = "rpc-client")]
pub use self::prod::ProdEvidenceReporter;

#[cfg(feature = "rpc-client")]
mod prod {
    use std::{collections::HashMap, time::Duration};

    use contracts::requires;
    use tendermint_rpc as rpc;
    use tendermint_rpc::Client;

    use super::*;
    use crate::utils::block_on;

    /// Production implementation of the EvidenceReporter component, which reports evidence to full
    /// nodes via RPC.
    #[derive(Clone, Debug)]
    pub struct ProdEvidenceReporter {
        peer_map: HashMap<PeerId, tendermint_rpc::Url>,
        timeout: Option<Duration>,
    }

    #[contract_trait]
    impl EvidenceReporter for ProdEvidenceReporter {
        #[requires(self.peer_map.contains_key(&peer))]
        fn report(&self, e: Evidence, peer: PeerId) -> Result<Hash, IoError> {
            let client = self.rpc_client_for(peer)?;

            let response = block_on(
                self.timeout,
                async move { client.broadcast_evidence(e).await },
            )?
            .map_err(IoError::rpc)?;

            Ok(response.hash)
        }
    }

    impl ProdEvidenceReporter {
        /// Constructs a new ProdEvidenceReporter component.
        ///
        /// A peer map which maps peer IDS to their network address must be supplied.
        pub fn new(
            peer_map: HashMap<PeerId, tendermint_rpc::Url>,
            timeout: Option<Duration>,
        ) -> Self {
            Self { peer_map, timeout }
        }

        #[requires(self.peer_map.contains_key(&peer))]
        fn rpc_client_for(&self, peer: PeerId) -> Result<rpc::HttpClient, IoError> {
            let peer_addr = self.peer_map.get(&peer).unwrap().to_owned();
            rpc::HttpClient::new(peer_addr).map_err(IoError::rpc)
        }
    }
}
