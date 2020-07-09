use std::collections::HashMap;
use std::time::Duration;

use contracts::{contract_trait, post, pre};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use tendermint::{
    block::signed_header::SignedHeader as TMSignedHeader, validator::Set as TMValidatorSet,
};

use tendermint_rpc as rpc;

use crate::{
    bail,
    types::{Height, LightBlock, PeerId, TMLightBlock},
};

pub enum AtHeight {
    At(Height),
    Highest,
}

impl From<Height> for AtHeight {
    fn from(height: Height) -> Self {
        if height == 0 {
            Self::Highest
        } else {
            Self::At(height)
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Serialize, Deserialize)]
pub enum IoError {
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
pub trait Io<LB>: Send
where
    LB: LightBlock,
{
    /// Fetch a light block at the given height from the peer with the given peer ID.
    ///
    /// ## Postcondition
    /// - The provider of the returned light block matches the given peer [LCV-IO-POST-PROVIDER]
    #[post(ret.as_ref().map(|lb| lb.provider() == peer).unwrap_or(true))]
    fn fetch_light_block(&self, peer: PeerId, height: AtHeight) -> Result<LB, IoError>;
}

#[contract_trait]
impl<F: Send, LB: LightBlock> Io<LB> for F
where
    F: Fn(PeerId, AtHeight) -> Result<LB, IoError>,
{
    fn fetch_light_block(&self, peer: PeerId, height: AtHeight) -> Result<LB, IoError> {
        self(peer, height)
    }
}

/// Production implementation of the Io component, which fetches
/// light blocks from full nodes via RPC.
#[derive(Clone, Debug)]
pub struct ProdIo {
    peer_map: HashMap<PeerId, tendermint::net::Address>,
    timeout: Option<Duration>,
}

#[contract_trait]
impl Io<TMLightBlock> for ProdIo {
    fn fetch_light_block(&self, peer: PeerId, height: AtHeight) -> Result<TMLightBlock, IoError> {
        let signed_header = self.fetch_signed_header(peer, height)?;
        let height: Height = signed_header.header.height.into();

        let validator_set = self.fetch_validator_set(peer, height.into())?;
        let next_validator_set = self.fetch_validator_set(peer, (height + 1).into())?;

        let light_block = TMLightBlock::new(signed_header, validator_set, next_validator_set, peer);

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
        let rpc_client = self.rpc_client_for(peer);

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
            self.rpc_client_for(peer).validators(height),
            peer,
            self.timeout,
        )?;

        match res {
            Ok(response) => Ok(TMValidatorSet::new(response.validators)),
            Err(err) => Err(IoError::IoError(err)),
        }
    }

    // FIXME: Cannot enable precondition because of "autoref lifetime" issue
    // #[pre(self.peer_map.contains_key(&peer))]
    fn rpc_client_for(&self, peer: PeerId) -> rpc::Client {
        let peer_addr = self.peer_map.get(&peer).unwrap().to_owned();
        rpc::Client::new(peer_addr)
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
