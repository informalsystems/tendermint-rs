use serde::{Deserialize, Serialize};
use tendermint::{block, rpc};
use thiserror::Error;

use tendermint::block::signed_header::SignedHeader as TMSignedHeader;
use tendermint::validator::Set as TMValidatorSet;

use crate::prelude::*;
use std::collections::HashMap;

pub const LATEST_HEIGHT: Height = 0;

#[derive(Clone, Debug, Error, PartialEq, Serialize, Deserialize)]
pub enum IoError {
    #[error(transparent)]
    IoError(#[from] rpc::Error),
}

pub trait Io {
    fn fetch_light_block(&mut self, peer: PeerId, height: Height) -> Result<LightBlock, IoError>;
}

impl<F> Io for F
where
    F: FnMut(PeerId, Height) -> Result<LightBlock, IoError>,
{
    fn fetch_light_block(&mut self, peer: PeerId, height: Height) -> Result<LightBlock, IoError> {
        self(peer, height)
    }
}

pub struct ProdIo {
    rpc_clients: HashMap<PeerId, rpc::Client>,
    peer_map: HashMap<PeerId, tendermint::net::Address>,
}

impl Io for ProdIo {
    fn fetch_light_block(&mut self, peer: PeerId, height: Height) -> Result<LightBlock, IoError> {
        let signed_header = self.fetch_signed_header(&peer, height)?;
        let height = signed_header.header.height.into();

        let validator_set = self.fetch_validator_set(&peer, height)?;
        let next_validator_set = self.fetch_validator_set(&peer, height + 1)?;

        let light_block = LightBlock::new(
            signed_header,
            validator_set,
            next_validator_set,
            // peer
        );

        Ok(light_block)
    }
}

impl ProdIo {
    pub fn new(peer_map: HashMap<PeerId, tendermint::net::Address>) -> Self {
        Self {
            rpc_clients: HashMap::new(),
            peer_map,
        }
    }

    // #[pre(self.peer_map.contains_key(peer))]
    fn fetch_signed_header(
        &mut self,
        peer: &PeerId,
        height: Height,
    ) -> Result<TMSignedHeader, IoError> {
        let height: block::Height = height.into();
        let rpc_client = self.rpc_client_for(peer);

        let res = block_on(async {
            match height.value() {
                0 => rpc_client.latest_commit().await,
                _ => rpc_client.commit(height).await,
            }
        });

        match res {
            Ok(response) => Ok(response.signed_header),
            Err(err) => Err(IoError::IoError(err)),
        }
    }

    // #[pre(self.peer_map.contains_key(peer))]
    fn fetch_validator_set(
        &mut self,
        peer: &PeerId,
        height: Height,
    ) -> Result<TMValidatorSet, IoError> {
        let res = block_on(self.rpc_client_for(peer).validators(height));

        match res {
            Ok(response) => Ok(TMValidatorSet::new(response.validators)),
            Err(err) => Err(IoError::IoError(err)),
        }
    }

    // #[pre(self.peer_map.contains_key(peer))]
    fn rpc_client_for(&mut self, peer: &PeerId) -> &mut rpc::Client {
        let peer_addr = self.peer_map.get(peer).unwrap().to_owned();
        self.rpc_clients
            .entry(peer.to_owned())
            .or_insert_with(|| rpc::Client::new(peer_addr))
    }
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}
