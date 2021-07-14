//! Provides an interface and a default implementation of the `Io` component

use flex_error::{define_error, TraceError};
use std::time::Duration;

#[cfg(feature = "rpc-client")]
use tendermint_rpc::Client;

use tendermint_rpc as rpc;

use crate::types::{Height, LightBlock};

#[cfg(feature = "tokio")]
type TimeoutError = flex_error::DisplayOnly<tokio::time::error::Elapsed>;

#[cfg(not(feature = "tokio"))]
type TimeoutError = flex_error::NoSource;

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
            [ rpc::Error ]
            | _ | { "rpc error" },

        InvalidHeight
            | _ | {
                "invalid height: given height must be greater than 0"
            },

        InvalidValidatorSet
            [ tendermint::Error ]
            | _ | { "fetched validator set is invalid" },

        Timeout
            { duration: Duration }
            [ TimeoutError ]
            | e | {
                format_args!("task timed out after {} ms",
                    e.duration.as_millis())
            },

        Runtime
            [ TraceError<std::io::Error> ]
            | _ | { "failed to initialize runtime" },

    }
}

impl IoErrorDetail {
    /// Whether this error means that a timeout occured when querying a node.
    pub fn is_timeout(&self) -> Option<Duration> {
        match self {
            Self::Timeout(e) => Some(e.duration),
            _ => None,
        }
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
                Err(err) => Err(rpc_error(err)),
            }
        }

        fn fetch_validator_set(
            &self,
            height: AtHeight,
            proposer_address: Option<TMAccountId>,
        ) -> Result<TMValidatorSet, IoError> {
            let height = match height {
                AtHeight::Highest => {
                    return Err(invalid_height_error());
                }
                AtHeight::At(height) => height,
            };

            let client = self.rpc_client.clone();
            let response = block_on(self.timeout, async move {
                client.validators(height, Paging::All).await
            })?
            .map_err(rpc_error)?;

            let validator_set = match proposer_address {
                Some(proposer_address) => {
                    TMValidatorSet::with_proposer(response.validators, proposer_address)
                        .map_err(invalid_validator_set_error)?
                }
                None => TMValidatorSet::without_proposer(response.validators),
            };

            Ok(validator_set)
        }
    }
}
