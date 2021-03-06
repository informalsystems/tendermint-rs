//! Provides an interface and a default implementation of the `Io` component

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[cfg(feature = "rpc-client")]
use tendermint_rpc::Client;

use tendermint_rpc as rpc;

use crate::types::{Height, LightBlock};

/// Type for selecting either a specific height or the latest one
pub enum AtHeight {
    /// A specific height
    At(Height),
    /// The latest height
    Highest,
}

impl From<Height> for AtHeight {
    fn from(height: Height) -> Self {
        if height.value() == 0 {
            Self::Highest
        } else {
            Self::At(height)
        }
    }
}

/// I/O errors
#[derive(Clone, Debug, Error, PartialEq, Serialize, Deserialize)]
pub enum IoError {
    /// Wrapper for a `tendermint::rpc::Error`.
    #[error(transparent)]
    RpcError(#[from] rpc::Error),

    /// Given height is invalid
    #[error("invalid height: {0}")]
    InvalidHeight(String),

    /// Fetched validator set is invalid
    #[error("fetched validator set is invalid: {0}")]
    InvalidValidatorSet(String),

    /// Task timed out.
    #[error("task timed out after {} ms", .0.as_millis())]
    Timeout(Duration),

    /// Failed to initialize runtime
    #[error("failed to initialize runtime")]
    Runtime,
}

impl IoError {
    /// Whether this error means that a timeout occured when querying a node.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }
}

/// Interface for fetching light blocks from a full node, typically via the RPC client.
pub trait Io: Send + Sync {
    /// Fetch a light block at the given height from a peer
    fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, IoError>;
}

impl<F: Send + Sync> Io for F
where
    F: Fn(AtHeight) -> Result<LightBlock, IoError>,
{
    fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, IoError> {
        self(height)
    }
}

#[cfg(feature = "rpc-client")]
pub use self::prod::ProdIo;

#[cfg(feature = "rpc-client")]
mod prod {
    use super::*;

    use std::time::Duration;

    use crate::bail;
    use crate::types::PeerId;
    use crate::utils::block_on;

    use tendermint::account::Id as TMAccountId;
    use tendermint::block::signed_header::SignedHeader as TMSignedHeader;
    use tendermint::validator::Set as TMValidatorSet;
    use tendermint_rpc::Paging;

    /// Production implementation of the Io component, which fetches
    /// light blocks from full nodes via RPC.
    #[derive(Clone, Debug)]
    pub struct ProdIo {
        peer_id: PeerId,
        rpc_client: rpc::HttpClient,
        timeout: Option<Duration>,
    }

    impl Io for ProdIo {
        fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, IoError> {
            let signed_header = self.fetch_signed_header(height)?;
            let height = signed_header.header.height;
            let proposer_address = signed_header.header.proposer_address;

            let validator_set = self.fetch_validator_set(height.into(), Some(proposer_address))?;
            let next_validator_set = self.fetch_validator_set(height.increment().into(), None)?;

            let light_block = LightBlock::new(
                signed_header,
                validator_set,
                next_validator_set,
                self.peer_id,
            );

            Ok(light_block)
        }
    }

    impl ProdIo {
        /// Constructs a new ProdIo component.
        ///
        /// A peer map which maps peer IDS to their network address must be supplied.
        pub fn new(
            peer_id: PeerId,
            rpc_client: rpc::HttpClient, /* TODO(thane): Generalize over client transport
                                          * (instead of using HttpClient directly) */
            timeout: Option<Duration>,
        ) -> Self {
            Self {
                peer_id,
                rpc_client,
                timeout,
            }
        }

        fn fetch_signed_header(&self, height: AtHeight) -> Result<TMSignedHeader, IoError> {
            let client = self.rpc_client.clone();
            let res = block_on(self.timeout, async move {
                match height {
                    AtHeight::Highest => client.latest_commit().await,
                    AtHeight::At(height) => client.commit(height).await,
                }
            })?;

            match res {
                Ok(response) => Ok(response.signed_header),
                Err(err) => Err(IoError::RpcError(err)),
            }
        }

        fn fetch_validator_set(
            &self,
            height: AtHeight,
            proposer_address: Option<TMAccountId>,
        ) -> Result<TMValidatorSet, IoError> {
            let height = match height {
                AtHeight::Highest => bail!(IoError::InvalidHeight(
                    "given height must be greater than 0".to_string()
                )),
                AtHeight::At(height) => height,
            };

            let client = self.rpc_client.clone();
            let response = block_on(self.timeout, async move {
                client.validators(height, Paging::All).await
            })?
            .map_err(IoError::RpcError)?;

            let validator_set = match proposer_address {
                Some(proposer_address) => {
                    TMValidatorSet::with_proposer(response.validators, proposer_address)
                        .map_err(|e| IoError::InvalidValidatorSet(e.to_string()))?
                }
                None => TMValidatorSet::without_proposer(response.validators),
            };

            Ok(validator_set)
        }
    }
}
