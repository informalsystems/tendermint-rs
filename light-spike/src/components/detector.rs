use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prelude::*;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForkDetectorError {
    #[error("conflicting blocks {:?} {:?}")]
    ConflictingBlocks(LightBlock, LightBlock),
}

impl_event!(VerifierError);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForkDetectorInput {
    Detect(Vec<LightBlock>),
}

impl_event!(ForkDetectorInput);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForkDetectorOutput {
    Detected(LightBlock, LightBlock)
    NotDetected(),
}

impl_event!(ForkDetectorOutput);

pub struct ForkDetector {
    header_hasher: Box<dyn HeaderHasher>,
}

impl ForkDetector {
    pub fn new( header_hasher: impl HeaderHasher + 'static) -> Self {
       Self { header_hasher: Box::new(header_hasher) }
    }

    pub fn detect(&self, light_blocks: Vec<LightBlock>) -> Result<_, ConflictingBlocks> {
        if light_blocks.is_empty() {
            return Ok()
        }

        let first_block = light_blocks.pop()
        let hash = first_block.expect("light_blocks is empty").hash()
        for lb in light_blocks.into_iter() {
            if hash != lb.hash() {
                return ConflictingBlocks{first_block, lb}
            }
        }

        return Ok()
    }
}
