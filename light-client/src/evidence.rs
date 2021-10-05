//! Fork evidence data structures and interfaces.

use async_trait::async_trait;

pub use tendermint::evidence::Evidence;
use tendermint_rpc::abci::transaction::Hash;

use crate::{components::io::IoError, types::PeerId};

/// Interface for reporting evidence to full nodes, typically via the RPC client.
#[allow(missing_docs)] // This is required because of the `contracts` crate (TODO: open/link issue)
#[async_trait]
pub trait EvidenceReporter: Send + Sync {
    /// Report evidence to all connected full nodes.
    async fn report(&self, e: Evidence, peer: PeerId) -> Result<Hash, IoError>;
}

#[cfg(feature = "rpc-client")]
pub use self::prod::ProdEvidenceReporter;

#[cfg(feature = "rpc-client")]
mod prod {
    use std::{collections::HashMap, time::Duration};

    use contracts::requires;
    use futures::future::FutureExt as _;

    use tendermint_rpc as rpc;
    use tendermint_rpc::Client;

    use super::*;

    use crate::utils::time::timeout;

    /// Production implementation of the EvidenceReporter component, which reports evidence to full
    /// nodes via RPC.
    #[derive(Clone, Debug)]
    pub struct ProdEvidenceReporter {
        peer_map: HashMap<PeerId, tendermint_rpc::Url>,
        timeout: Duration,
    }

    #[async_trait]
    impl EvidenceReporter for ProdEvidenceReporter {
        async fn report(&self, e: Evidence, peer: PeerId) -> Result<Hash, IoError> {
            let client = self.rpc_client_for(peer)?;

            let response = timeout(self.timeout, client.broadcast_evidence(e).fuse())
                .await
                .map_err(IoError::time)?
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
            Self {
                peer_map,
                timeout: timeout.unwrap_or_else(|| Duration::from_secs(5)),
            }
        }

        #[requires(self.peer_map.contains_key(&peer))]
        fn rpc_client_for(&self, peer: PeerId) -> Result<rpc::HttpClient, IoError> {
            let peer_addr = self.peer_map.get(&peer).unwrap().to_owned();
            rpc::HttpClient::new(peer_addr).map_err(IoError::rpc)
        }
    }
}
