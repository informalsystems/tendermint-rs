use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ForkDetection {
    Detected(LightBlock, LightBlock),
    NotDetected,
}

pub trait ForkDetector {
    fn detect(&self, light_blocks: Vec<LightBlock>) -> ForkDetection;
}

pub struct RealForkDetector {
    header_hasher: Box<dyn HeaderHasher>,
}

impl RealForkDetector {
    pub fn new(header_hasher: impl HeaderHasher + 'static) -> Self {
        Self {
            header_hasher: Box::new(header_hasher),
        }
    }
}

impl ForkDetector for RealForkDetector {
    fn detect(&self, mut light_blocks: Vec<LightBlock>) -> ForkDetection {
        if light_blocks.is_empty() {
            return ForkDetection::NotDetected;
        }

        let first_block = light_blocks.pop().unwrap(); // Safety: checked above that not empty.
        let first_hash = self.header_hasher.hash(&first_block.signed_header.header);

        for light_block in light_blocks {
            let hash = self.header_hasher.hash(&light_block.signed_header.header);

            if first_hash != hash {
                return ForkDetection::Detected(first_block, light_block);
            }
        }

        ForkDetection::NotDetected
    }
}
