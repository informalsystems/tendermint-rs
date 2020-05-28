use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ForkDetection {
    // NOTE: We box the fields to reduce the overall size of the enum.
    //       See https://rust-lang.github.io/rust-clippy/master/index.html#large_enum_variant
    Detected(Vec<Box<LightBlock>>),
    NotDetected,
}

pub trait ForkDetector {
    fn detect(&self, verified_block: &LightBlock, light_blocks: Vec<LightBlock>) -> ForkDetection;
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
    fn detect(&self, verified_block: &LightBlock, light_blocks: Vec<LightBlock>) -> ForkDetection {
        let first_hash = self.header_hasher.hash(&verified_block.signed_header.header);

        for light_block in light_blocks {
            let hash = self.header_hasher.hash(&light_block.signed_header.header);

            if first_hash != hash {
                // TODO: use verifier to see if light_block verifies
                return ForkDetection::Detected(vec![Box::new(light_block)]);
            }
        }

        ForkDetection::NotDetected
    }
}
