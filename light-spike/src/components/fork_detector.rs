use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ForkDetectorInput {
    Detect(Vec<LightBlock>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ForkDetectorOutput {
    Detected(LightBlock, LightBlock),
    NotDetected,
}

pub trait ForkDetector {
    fn process(&self, input: ForkDetectorInput) -> ForkDetectorOutput;
}

pub struct RealForkDetector {
    header_hasher: Box<dyn HeaderHasher>,
}

impl ForkDetector for RealForkDetector {
    fn process(&self, input: ForkDetectorInput) -> ForkDetectorOutput {
        match input {
            ForkDetectorInput::Detect(light_blocks) => self.detect(light_blocks),
        }
    }
}

impl RealForkDetector {
    pub fn new(header_hasher: impl HeaderHasher + 'static) -> Self {
        Self {
            header_hasher: Box::new(header_hasher),
        }
    }

    pub fn detect(&self, mut light_blocks: Vec<LightBlock>) -> ForkDetectorOutput {
        if light_blocks.is_empty() {
            return ForkDetectorOutput::NotDetected;
        }

        let first_block = light_blocks.pop().unwrap(); // Safety: checked above that not empty.
        let first_hash = self.header_hasher.hash(first_block.header());

        for light_block in light_blocks {
            let hash = self.header_hasher.hash(light_block.header());

            if first_hash != hash {
                return ForkDetectorOutput::Detected(first_block, light_block);
            }
        }

        ForkDetectorOutput::NotDetected
    }
}
