use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::future::Future;
use std::sync::mpsc::Sender;
use tendermint::{block, rpc};

use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum RequesterError {
    RpcError(rpc::Error),
}

#[typetag::serde]
impl Event for RequesterError {}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequesterInput {
    FetchState(Height),
}

#[typetag::serde]
impl Event for RequesterInput {}

#[derive(Debug, Serialize, Deserialize)]
pub enum RequesterOutput {
    FetchedLightBlock(LightBlock),
}

#[typetag::serde]
impl Event for RequesterOutput {}

pub struct Requester {
    rpc_client: rpc::Client,
    trace: Sender<BoxedEvent>,
}

impl Requester {
    pub fn new(rpc_client: rpc::Client, trace: Sender<BoxedEvent>) -> Self {
        Self { rpc_client, trace }
    }

    fn trace(&self, e: impl Event + 'static) {
        self.trace.send(Box::new(e)).expect("could not trace event");
    }

    pub fn fetch_light_block(&self, height: Height) -> Result<LightBlock, RequesterError> {
        self.trace(RequesterInput::FetchState(height));

        let signed_header = self.fetch_signed_header(height)?;
        let validator_set = self.fetch_validator_set(height)?;
        let next_validator_set = self.fetch_validator_set(height + 1)?;

        let light_block = LightBlock {
            height,
            signed_header,
            validator_set,
            next_validator_set,
        };

        self.trace(RequesterOutput::FetchedLightBlock(light_block.clone()));

        Ok(light_block)
    }

    fn fetch_signed_header(&self, h: Height) -> Result<SignedHeader, RequesterError> {
        let height: block::Height = h.into();

        let res = block_on(async {
            match height.value() {
                0 => self.rpc_client.latest_commit().await,
                _ => self.rpc_client.commit(height).await,
            }
        });

        match res {
            Ok(response) => Ok(response.signed_header.into()),
            Err(err) => Err(RequesterError::RpcError(err)),
        }
    }

    fn fetch_validator_set(&self, height: Height) -> Result<ValidatorSet, RequesterError> {
        let res = block_on(self.rpc_client.validators(height));

        match res {
            Ok(response) => Ok(response.validators.into()),
            Err(err) => Err(RequesterError::RpcError(err)),
        }
    }
}

fn block_on<F: Future>(_future: F) -> F::Output {
    todo!()
}

