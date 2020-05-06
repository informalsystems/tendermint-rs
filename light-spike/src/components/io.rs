use serde::{Deserialize, Serialize};
use tendermint::{block, rpc};
use thiserror::Error;

use tendermint::block::signed_header::SignedHeader as TMSignedHeader;
// use tendermint::lite::types::Height as _;

use crate::prelude::*;
use std::collections::HashMap;

pub const LATEST_HEIGHT: Height = 0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IoInput {
    FetchLightBlock { peer: Peer, height: Height },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IoOutput {
    FetchedLightBlock(LightBlock),
}

#[derive(Clone, Debug, Error, PartialEq, Serialize, Deserialize)]
pub enum IoError {
    #[error(transparent)]
    IoError(#[from] rpc::Error),
}

pub type IoResult = Result<IoOutput, IoError>;

pub trait Io {
    fn process(&mut self, input: IoInput) -> IoResult;
}

impl<F> Io for F
where
    F: FnMut(IoInput) -> IoResult,
{
    fn process(&mut self, input: IoInput) -> IoResult {
        self(input)
    }
}

pub struct RealIo {
    rpc_clients: HashMap<Peer, rpc::Client>,
}

impl Io for RealIo {
    fn process(&mut self, input: IoInput) -> IoResult {
        match input {
            IoInput::FetchLightBlock { peer, height } => self.fetch_light_block(peer, height),
        }
    }
}

impl RealIo {
    pub fn new() -> Self {
        Self {
            rpc_clients: HashMap::new(),
        }
    }

    pub fn fetch_light_block(&mut self, peer: Peer, height: Height) -> IoResult {
        let signed_header = self.fetch_signed_header(peer.clone(), height)?;
        let light_block = LightBlock::from_signed_header(signed_header, peer);

        Ok(IoOutput::FetchedLightBlock(light_block))
    }

    fn fetch_signed_header(
        &mut self,
        peer: Peer,
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

    fn rpc_client_for(&mut self, peer: Peer) -> &mut rpc::Client {
        self.rpc_clients
            .entry(peer.clone())
            .or_insert_with(|| rpc::Client::new(peer))
    }

    // fn fetch_validator_set(&self, height: Height) -> Result<TMValidatorSet, IoError> {
    //     let res = block_on(self.rpc_client.validators(height));

    //     match res {
    //         Ok(response) => Ok(TMValidatorSet::new(response.validators)),
    //         Err(err) => Err(IoError::IoError(err)),
    //     }
    // }
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}
