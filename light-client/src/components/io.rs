//! Provides an interface and a default implementation of the `Io` component

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "rpc-client")]
use tendermint_rpc::Client;

use tendermint_rpc as rpc;

use crate::types::{Height, LightBlock, PeerId};

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

    /// The request timed out.
    #[error("request to peer {0} timed out")]
    Timeout(PeerId),
}

impl IoError {
    /// Whether this error means that a timeout occured when querying a node.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }
}

/// Interface for fetching light blocks from a full node, typically via the RPC client.
#[allow(missing_docs)] // This is required because of the `contracts` crate (TODO: open/link issue)
pub trait Io: Send {
    /// Fetch a light block at the given height from a peer
    fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, IoError>;
}

impl<F: Send> Io for F
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

    use crate::{bail, utils::block_on};
    use tendermint::block::signed_header::SignedHeader as TMSignedHeader;
    use tendermint::validator::Set as TMValidatorSet;

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

            let validator_set = self.fetch_validator_set(height.into())?;
            let next_validator_set = self.fetch_validator_set(height.increment().into())?;

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
            let mut client = self.rpc_client.clone();
            let res = block_on(
                async move {
                    match height {
                        AtHeight::Highest => client.latest_commit().await,
                        AtHeight::At(height) => client.commit(height).await,
                    }
                },
                self.peer_id,
                self.timeout,
            )?;

            match res {
                Ok(response) => Ok(response.signed_header),
                Err(err) => Err(IoError::RpcError(err)),
            }
        }

        fn fetch_validator_set(&self, height: AtHeight) -> Result<TMValidatorSet, IoError> {
            let height = match height {
                AtHeight::Highest => bail!(IoError::InvalidHeight(
                    "given height must be greater than 0".to_string()
                )),
                AtHeight::At(height) => height,
            };

            let mut client = self.rpc_client.clone();
            let task = async move { client.validators(height).await };
            let res = block_on(task, self.peer_id, self.timeout)?;

            match res {
                Ok(response) => Ok(TMValidatorSet::new(response.validators)),
                Err(err) => Err(IoError::RpcError(err)),
            }
        }
    }
}
