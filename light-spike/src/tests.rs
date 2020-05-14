use crate::prelude::*;

use serde::Deserialize;

use tendermint::block::Height as HeightStr;
use tendermint::evidence::Duration as DurationStr;

#[derive(Deserialize, Clone, Debug)]
pub struct TestCases {
    pub batch_name: String,
    pub test_cases: Vec<TestCase>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TestCase {
    pub description: String,
    pub initial: Initial,
    pub input: Vec<LightBlock>,
    pub expected_output: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Initial {
    pub signed_header: SignedHeader,
    pub next_validator_set: ValidatorSet,
    pub trusting_period: DurationStr,
    pub now: Time,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TestBisection {
    pub description: String,
    pub trust_options: TrustOptions,
    pub primary: Provider,
    pub height_to_verify: HeightStr,
    pub now: Time,
    pub expected_output: Option<String>,
    pub expected_num_of_bisections: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Provider {
    pub chain_id: String,
    pub lite_blocks: Vec<LightBlock>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TrustOptions {
    pub period: DurationStr,
    pub height: HeightStr,
    pub hash: Hash,
    pub trust_level: TrustThreshold,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Trusted {
    pub signed_header: SignedHeader,
    pub next_validators: ValidatorSet,
}

impl Trusted {
    pub fn new(signed_header: SignedHeader, next_validators: ValidatorSet) -> Self {
        Self {
            signed_header,
            next_validators,
        }
    }
}
