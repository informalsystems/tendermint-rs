use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prelude::*;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForkDetectorError {
    #[error("conflicting blocks: {0} and {1}")]
    ConflictingBlocks(LightBlock, LightBlock),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForkDetectorInput {
    Detect(Vec<LightBlock>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForkDetectorOutput {
    NotDetected,
}

pub type ForkDetectorResult = Result<ForkDetectorOutput, ForkDetectorError>;

pub struct ForkDetector {
    header_hasher: Box<dyn HeaderHasher>,
}

impl ForkDetector {
    pub fn new(header_hasher: impl HeaderHasher + 'static) -> Self {
        Self {
            header_hasher: Box::new(header_hasher),
        }
    }

    pub fn process(&self, input: ForkDetectorInput) -> ForkDetectorResult {
        match input {
            ForkDetectorInput::Detect(light_blocks) => self.detect(light_blocks),
        }
    }

    pub fn detect(&self, mut light_blocks: Vec<LightBlock>) -> ForkDetectorResult {
        if light_blocks.is_empty() {
            return Ok(ForkDetectorOutput::NotDetected);
        }

        let first_block = light_blocks.pop().unwrap(); // Safety: checked above that not empty.
        let first_hash = self.header_hasher.hash(first_block.header());

        for light_block in light_blocks {
            let hash = self.header_hasher.hash(light_block.header());

            if first_hash != hash {
                return Err(ForkDetectorError::ConflictingBlocks(
                    first_block,
                    light_block,
                ));
            }
        }

        Ok(ForkDetectorOutput::NotDetected)
    }
}
