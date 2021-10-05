//! Provides an interface and a default implementation of the `Io` component

use std::time::Duration;

use async_trait::async_trait;
use flex_error::{define_error, TraceError};

use crate::utils::time::TimeError;
use crate::verifier::types::{Height, LightBlock};

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

define_error! {
    #[derive(Debug)]
    IoError {
        Rpc
            [ tendermint_rpc::Error ]
            | _ | { "rpc error" },

        InvalidHeight
            | _ | {
                "invalid height: given height must be greater than 0"
            },

        InvalidValidatorSet
            [ tendermint::Error ]
            | _ | { "fetched validator set is invalid" },

        Time
            [ TimeError ]
            | _ | { "time error" },

        Runtime
            [ TraceError<std::io::Error> ]
            | _ | { "failed to initialize runtime" },

    }
}

impl IoErrorDetail {
    /// Whether this error means that a timeout occured when querying a node.
    pub fn is_timeout(&self) -> Option<Duration> {
        match self {
            Self::Time(e) => e.source.is_timeout(),
            _ => None,
        }
    }
}

#[async_trait]
pub trait AsyncIo: Send + Sync {
    async fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, IoError>;
}

#[async_trait]
impl<F, R> AsyncIo for F
where
    F: Fn(AtHeight) -> R + Send + Sync,
    R: std::future::Future<Output = Result<LightBlock, IoError>> + Send,
{
    async fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, IoError> {
        self(height).await
    }
}

#[cfg(feature = "rpc-client")]
pub use self::rpc::RpcIo;

#[cfg(feature = "rpc-client")]
mod rpc {
    use std::time::Duration;

    use futures::future::FutureExt;
    use tendermint::account::Id as TMAccountId;
    use tendermint::block::signed_header::SignedHeader as TMSignedHeader;
    use tendermint::validator::Set as TMValidatorSet;
    use tendermint_rpc::{Client as _, Paging};

    use crate::utils::time::timeout;
    use crate::verifier::types::PeerId;

    use super::*;

    /// Implementation of the Io component backed by an RPC client, which fetches
    /// light blocks from full nodes.
    #[derive(Clone, Debug)]
    pub struct RpcIo {
        peer_id: PeerId,
        rpc_client: tendermint_rpc::HttpClient,
        timeout: Duration,
    }

    #[async_trait]
    impl AsyncIo for RpcIo {
        async fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, IoError> {
            let signed_header = self.fetch_signed_header(height).await?;
            let height = signed_header.header.height;
            let proposer_address = signed_header.header.proposer_address;

            let validator_set = self
                .fetch_validator_set(height.into(), Some(proposer_address))
                .await?;
            let next_validator_set = self
                .fetch_validator_set(height.increment().into(), None)
                .await?;

            let light_block = LightBlock::new(
                signed_header,
                validator_set,
                next_validator_set,
                self.peer_id,
            );

            Ok(light_block)
        }
    }

    impl RpcIo {
        /// Constructs a new RpcIo component.
        ///
        /// A peer map which maps peer IDS to their network address must be supplied.
        pub fn new(
            peer_id: PeerId,
            rpc_client: tendermint_rpc::HttpClient, /* TODO(thane): Generalize over client transport
                                                     * (instead of using HttpClient directly) */
            timeout: Option<Duration>,
        ) -> Self {
            Self {
                peer_id,
                rpc_client,
                timeout: timeout.unwrap_or_else(|| Duration::from_secs(5)),
            }
        }

        async fn fetch_signed_header(&self, height: AtHeight) -> Result<TMSignedHeader, IoError> {
            let client = self.rpc_client.clone();
            let fetch_commit = match height {
                AtHeight::Highest => client.latest_commit().fuse(),
                AtHeight::At(height) => client.commit(height).fuse(),
            };
            let res = timeout(self.timeout, fetch_commit)
                .await
                .map_err(IoError::time)?;

            match res {
                Ok(response) => Ok(response.signed_header),
                Err(err) => Err(IoError::rpc(err)),
            }
        }

        async fn fetch_validator_set(
            &self,
            height: AtHeight,
            proposer_address: Option<TMAccountId>,
        ) -> Result<TMValidatorSet, IoError> {
            let height = match height {
                AtHeight::Highest => {
                    return Err(IoError::invalid_height());
                },
                AtHeight::At(height) => height,
            };

            let client = self.rpc_client.clone();
            let response = timeout(self.timeout, client.validators(height, Paging::All).fuse())
                .await
                .map_err(IoError::time)?
                .map_err(IoError::rpc)?;

            let validator_set = match proposer_address {
                Some(proposer_address) => {
                    TMValidatorSet::with_proposer(response.validators, proposer_address)
                        .map_err(IoError::invalid_validator_set)?
                },
                None => TMValidatorSet::without_proposer(response.validators),
            };

            Ok(validator_set)
        }
    }
}
