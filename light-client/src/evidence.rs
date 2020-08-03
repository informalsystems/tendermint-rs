//! Fork evidence data structures and interfaces.

use crate::{components::io::IoError, types::PeerId};

use tendermint::abci::transaction::Hash;
use tendermint_rpc as rpc;

use async_trait::async_trait;
use contracts::{contract_trait, pre};
use std::collections::HashMap;

pub use tendermint::evidence::Evidence;

/// Interface for reporting evidence to full nodes, typically via the RPC client.
#[async_trait]
#[contract_trait]
#[allow(missing_docs)] // This is required because of the `contracts` crate (TODO: open/link issue)
pub trait EvidenceReporter: Sync + Send {
    /// Report evidence to all connected full nodes.
    async fn report(&self, e: Evidence, peer: PeerId) -> Result<Hash, IoError>;
}

/// Production implementation of the EvidenceReporter component, which reports evidence to full
/// nodes via RPC.
#[derive(Clone, Debug)]
pub struct ProdEvidenceReporter {
    peer_map: HashMap<PeerId, tendermint::net::Address>,
}

#[async_trait]
#[contract_trait]
impl EvidenceReporter for ProdEvidenceReporter {
    #[pre(self.peer_map.contains_key(&peer))]
    async fn report(&self, e: Evidence, peer: PeerId) -> Result<Hash, IoError> {
        let res = self.rpc_client_for(peer).broadcast_evidence(e).await;

        match res {
            Ok(response) => Ok(response.hash),
            Err(err) => Err(IoError::IoError(err)),
        }
    }
}

impl ProdEvidenceReporter {
    /// Constructs a new ProdEvidenceReporter component.
    ///
    /// A peer map which maps peer IDS to their network address must be supplied.
    pub fn new(peer_map: HashMap<PeerId, tendermint::net::Address>) -> Self {
        Self { peer_map }
    }

    // FIXME: Cannot enable precondition because of "autoref lifetime" issue
    // #[pre(self.peer_map.contains_key(&peer))]
    fn rpc_client_for(&self, peer: PeerId) -> rpc::Client {
        let peer_addr = self.peer_map.get(&peer).unwrap().to_owned();
        rpc::Client::new(peer_addr)
    }
}

