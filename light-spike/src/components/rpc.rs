use std::fmt::Debug;
use std::future::Future;
use std::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};
use tendermint::{block, rpc};
use thiserror::Error;

use crate::prelude::*;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcError {
    #[error(transparent)]
    RpcError(#[from] rpc::Error),
}

impl_event!(RpcError);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcInput {
    FetchState(Height),
}

impl_event!(RpcInput);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcOutput {
    FetchedLightBlock(LightBlock),
}

impl_event!(RpcOutput);

pub struct Rpc {
    rpc_client: rpc::Client,
    trace: Sender<BoxedEvent>,
}

impl Rpc {
    pub fn new(rpc_client: rpc::Client, trace: Sender<BoxedEvent>) -> Self {
        Self { rpc_client, trace }
    }

    fn trace(&self, e: impl Event + 'static) {
        self.trace.send(Box::new(e)).expect("could not trace event");
    }

    pub fn fetch_light_block(&self, height: Height) -> Result<LightBlock, RpcError> {
        self.trace(RpcInput::FetchState(height));

        let signed_header = self.fetch_signed_header(height)?;
        let validator_set = self.fetch_validator_set(height)?;
        let next_validator_set = self.fetch_validator_set(height + 1)?;

        let light_block = LightBlock {
            height,
            signed_header,
            validator_set,
            next_validator_set,
        };

        self.trace(RpcOutput::FetchedLightBlock(light_block.clone()));

        Ok(light_block)
    }

    fn fetch_signed_header(&self, h: Height) -> Result<SignedHeader, RpcError> {
        let height: block::Height = h.into();

        let res = block_on(async {
            match height.value() {
                0 => self.rpc_client.latest_commit().await,
                _ => self.rpc_client.commit(height).await,
            }
        });

        match res {
            Ok(response) => Ok(response.signed_header.into()),
            Err(err) => Err(RpcError::RpcError(err)),
        }
    }

    fn fetch_validator_set(&self, height: Height) -> Result<ValidatorSet, RpcError> {
        let res = block_on(self.rpc_client.validators(height));

        match res {
            Ok(response) => Ok(response.validators.into()),
            Err(err) => Err(RpcError::RpcError(err)),
        }
    }
}

fn block_on<F: Future>(_future: F) -> F::Output {
    todo!()
}
