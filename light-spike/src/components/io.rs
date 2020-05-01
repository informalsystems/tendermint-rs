use futures::executor::block_on;

use serde::{Deserialize, Serialize};
use tendermint::{block, rpc};
use thiserror::Error;

use crate::prelude::*;

pub const LATEST_HEIGHT: Height = 0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IoInput {
    FetchLightBlock(Height),
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
    fn process(&self, input: IoInput) -> IoResult;
}

impl<F> Io for F
where
    F: Fn(IoInput) -> IoResult,
{
    fn process(&self, input: IoInput) -> IoResult {
        self(input)
    }
}

pub struct RealIo {
    rpc_client: rpc::Client,
}

impl Io for RealIo {
    fn process(&self, input: IoInput) -> IoResult {
        match input {
            IoInput::FetchLightBlock(height) => self.fetch_light_block(height),
        }
    }
}

impl RealIo {
    pub fn new(rpc_client: rpc::Client) -> Self {
        Self { rpc_client }
    }

    pub fn fetch_light_block(&self, height: Height) -> IoResult {
        let signed_header = self.fetch_signed_header(height)?;
        let validators = self.fetch_validator_set(height)?;
        let next_validators = self.fetch_validator_set(height + 1)?;

        let light_block = LightBlock {
            height,
            signed_header,
            validators,
            next_validators,
        };

        Ok(IoOutput::FetchedLightBlock(light_block))
    }

    fn fetch_signed_header(&self, h: Height) -> Result<SignedHeader, IoError> {
        let height: block::Height = h.into();

        let res = block_on(async {
            match height.value() {
                0 => self.rpc_client.latest_commit().await,
                _ => self.rpc_client.commit(height).await,
            }
        });

        match res {
            Ok(response) => Ok(response.signed_header.into()),
            Err(err) => Err(IoError::IoError(err)),
        }
    }

    fn fetch_validator_set(&self, height: Height) -> Result<ValidatorSet, IoError> {
        let res = block_on(self.rpc_client.validators(height));

        match res {
            Ok(response) => Ok(response.validators.into()),
            Err(err) => Err(IoError::IoError(err)),
        }
    }
}
