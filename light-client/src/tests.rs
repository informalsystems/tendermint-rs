//! Utilities and datatypes for use in tests.

use crate::types::{Hash, LightBlock, PeerId, SignedHeader, Time, TrustThreshold, ValidatorSet};

use serde::Deserialize;

use tendermint::block::Height as HeightStr;
use tendermint::evidence::Duration as DurationStr;

#[derive(Deserialize, Clone, Debug)]
pub struct TestCases<LB> {
    pub batch_name: String,
    pub test_cases: Vec<TestCase<LB>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TestCase<LB> {
    pub description: String,
    pub initial: Initial,
    pub input: Vec<LB>,
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
pub struct TestBisection<LB> {
    pub description: String,
    pub trust_options: TrustOptions,
    pub primary: Provider<LB>,
    pub height_to_verify: HeightStr,
    pub now: Time,
    pub expected_output: Option<String>,
    pub expected_num_of_bisections: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Provider<LB> {
    pub chain_id: String,
    pub lite_blocks: Vec<LB>,
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

// -----------------------------------------------------------------------------
// Everything below is a temporary workaround for the lack of `provider` field
// in the light blocks serialized in the JSON fixtures.
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct AnonLightBlock {
    pub signed_header: SignedHeader,
    #[serde(rename = "validator_set")]
    pub validators: ValidatorSet,
    #[serde(rename = "next_validator_set")]
    pub next_validators: ValidatorSet,
    #[serde(default = "default_peer_id")]
    pub provider: PeerId,
}

pub fn default_peer_id() -> PeerId {
    "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap()
}

impl From<AnonLightBlock> for LightBlock {
    fn from(alb: AnonLightBlock) -> Self {
        Self {
            signed_header: alb.signed_header,
            validators: alb.validators,
            next_validators: alb.next_validators,
            provider: alb.provider,
        }
    }
}

impl From<TestCase<AnonLightBlock>> for TestCase<LightBlock> {
    fn from(tc: TestCase<AnonLightBlock>) -> Self {
        Self {
            description: tc.description,
            initial: tc.initial,
            input: tc.input.into_iter().map(Into::into).collect(),
            expected_output: tc.expected_output,
        }
    }
}

impl From<TestCases<AnonLightBlock>> for TestCases<LightBlock> {
    fn from(tc: TestCases<AnonLightBlock>) -> Self {
        Self {
            batch_name: tc.batch_name,
            test_cases: tc.test_cases.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Provider<AnonLightBlock>> for Provider<LightBlock> {
    fn from(p: Provider<AnonLightBlock>) -> Self {
        Self {
            chain_id: p.chain_id,
            lite_blocks: p.lite_blocks.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TestBisection<AnonLightBlock>> for TestBisection<LightBlock> {
    fn from(tb: TestBisection<AnonLightBlock>) -> Self {
        Self {
            description: tb.description,
            trust_options: tb.trust_options,
            primary: tb.primary.into(),
            height_to_verify: tb.height_to_verify,
            now: tb.now,
            expected_output: tb.expected_output,
            expected_num_of_bisections: tb.expected_num_of_bisections,
        }
    }
}
