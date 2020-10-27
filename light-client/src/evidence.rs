//! Fork evidence data structures and interfaces.

use crate::components::io::IoError;

use tendermint::{abci::transaction::Hash, net};

pub use tendermint::evidence::Evidence;

/// Interface for reporting evidence to full nodes, typically via the RPC client.
pub trait EvidenceReporter: Send + Sync {
    /// Report evidence to all connected full nodes.
    fn report(&self, e: Evidence, provider: net::Address) -> Result<Hash, IoError>;
}

#[cfg(feature = "rpc-client")]
pub use self::prod::ProdEvidenceReporter;

#[cfg(feature = "rpc-client")]
mod prod {
    use super::*;
    use crate::utils::block_on;

    use std::time::Duration;

    use tendermint_rpc as rpc;
    use tendermint_rpc::Client;

    /// Production implementation of the EvidenceReporter component, which reports evidence to full
    /// nodes via RPC.
    #[derive(Clone, Debug)]
    pub struct ProdEvidenceReporter {
        timeout: Option<Duration>,
    }

    impl EvidenceReporter for ProdEvidenceReporter {
        fn report(&self, e: Evidence, address: net::Address) -> Result<Hash, IoError> {
            let client = self.rpc_client_for(address)?;

            let res = block_on(
                self.timeout,
                async move { client.broadcast_evidence(e).await },
            )?;

            match res {
                Ok(response) => Ok(response.hash),
                Err(err) => Err(IoError::RpcError(err)),
            }
        }
    }

    impl ProdEvidenceReporter {
        /// Constructs a new ProdEvidenceReporter component.
        ///
        /// A peer map which maps peer IDS to their network address must be supplied.
        pub fn new(timeout: Option<Duration>) -> Self {
            Self { timeout }
        }

        fn rpc_client_for(&self, address: net::Address) -> Result<rpc::HttpClient, IoError> {
            Ok(rpc::HttpClient::new(address).map_err(IoError::from)?)
        }
    }
}
