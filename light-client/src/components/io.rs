//! Provides an interface and a default implementation of the `Io` component

use contracts::{contract_trait, post};
use serde::{Deserialize, Serialize};
#[cfg(feature = "rpc-client")]
use tendermint_rpc as rpc;
#[cfg(feature = "rpc-client")]
use tendermint_rpc::Client;
use thiserror::Error;

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
    #[cfg(feature = "rpc-client")]
    /// Wrapper for a `tendermint::rpc::Error`.
    #[error(transparent)]
    IoError(#[from] rpc::Error),

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
#[contract_trait]
#[allow(missing_docs)] // This is required because of the `contracts` crate (TODO: open/link issue)
pub trait Io: Send {
    /// Fetch a light block at the given height from the peer with the given peer ID.
    ///
    /// ## Postcondition
    /// - The provider of the returned light block matches the given peer [LCV-IO-POST-PROVIDER]
    #[post(ret.as_ref().map(|lb| lb.provider == peer).unwrap_or(true))]
    fn fetch_light_block(&self, peer: PeerId, height: AtHeight) -> Result<LightBlock, IoError>;
}

#[contract_trait]
impl<F: Send> Io for F
where
    F: Fn(PeerId, AtHeight) -> Result<LightBlock, IoError>,
{
    fn fetch_light_block(&self, peer: PeerId, height: AtHeight) -> Result<LightBlock, IoError> {
        self(peer, height)
    }
}

#[cfg(feature = "rpc-client")]
pub use self::prod::ProdIo;

#[cfg(feature = "rpc-client")]
mod prod {
    use super::*;

    use std::collections::HashMap;
    use std::time::Duration;

    use crate::bail;
    use contracts::{contract_trait, pre};
    use tendermint::{
        block::signed_header::SignedHeader as TMSignedHeader, validator::Set as TMValidatorSet,
    };

    /// Production implementation of the Io component, which fetches
    /// light blocks from full nodes via RPC.
    #[derive(Clone, Debug)]
    pub struct ProdIo {
        peer_map: HashMap<PeerId, tendermint::net::Address>,
        timeout: Option<Duration>,
    }

    #[contract_trait]
    impl Io for ProdIo {
        fn fetch_light_block(&self, peer: PeerId, height: AtHeight) -> Result<LightBlock, IoError> {
            let signed_header = self.fetch_signed_header(peer, height)?;
            let height = signed_header.header.height;

            let validator_set = self.fetch_validator_set(peer, height.into())?;
            let next_validator_set = self.fetch_validator_set(peer, height.increment().into())?;

            let light_block =
                LightBlock::new(signed_header, validator_set, next_validator_set, peer);

            Ok(light_block)
        }
    }

    impl ProdIo {
        /// Constructs a new ProdIo component.
        ///
        /// A peer map which maps peer IDS to their network address must be supplied.
        pub fn new(
            peer_map: HashMap<PeerId, tendermint::net::Address>,
            timeout: Option<Duration>,
        ) -> Self {
            Self { peer_map, timeout }
        }

        #[pre(self.peer_map.contains_key(&peer))]
        fn fetch_signed_header(
            &self,
            peer: PeerId,
            height: AtHeight,
        ) -> Result<TMSignedHeader, IoError> {
            let rpc_client = self.rpc_client_for(peer)?;

            let res = block_on(
                async {
                    match height {
                        AtHeight::Highest => rpc_client.latest_commit().await,
                        AtHeight::At(height) => rpc_client.commit(height).await,
                    }
                },
                peer,
                self.timeout,
            )?;

            match res {
                Ok(response) => Ok(response.signed_header),
                Err(err) => Err(IoError::IoError(err)),
            }
        }

        #[pre(self.peer_map.contains_key(&peer))]
        fn fetch_validator_set(
            &self,
            peer: PeerId,
            height: AtHeight,
        ) -> Result<TMValidatorSet, IoError> {
            let height = match height {
                AtHeight::Highest => bail!(IoError::InvalidHeight(
                    "given height must be greater than 0".to_string()
                )),
                AtHeight::At(height) => height,
            };

            let res = block_on(
                self.rpc_client_for(peer)?.validators(height),
                peer,
                self.timeout,
            )?;

            match res {
                Ok(response) => Ok(TMValidatorSet::new(response.validators)),
                Err(err) => Err(IoError::IoError(err)),
            }
        }

        // TODO(thane): Generalize over client transport (instead of using HttpClient directly).
        #[pre(self.peer_map.contains_key(&peer))]
        fn rpc_client_for(&self, peer: PeerId) -> Result<rpc::HttpClient, IoError> {
            let peer_addr = self.peer_map.get(&peer).unwrap().to_owned();
            Ok(rpc::HttpClient::new(peer_addr).map_err(IoError::from)?)
        }
    }

    fn block_on<F: std::future::Future>(
        f: F,
        peer: PeerId,
        timeout: Option<Duration>,
    ) -> Result<F::Output, IoError> {
        let mut rt = tokio::runtime::Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();

        if let Some(timeout) = timeout {
            rt.block_on(async { tokio::time::timeout(timeout, f).await })
                .map_err(|_| IoError::Timeout(peer))
        } else {
            Ok(rt.block_on(f))
        }
    }
}
