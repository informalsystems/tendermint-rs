//! Utilities and datatypes for use in tests.

use std::{collections::HashMap, time::Duration};

use contracts::contract_trait;
use serde::{Deserialize, Serialize};
use tendermint::{
    block::Height as HeightStr,
    evidence::{Duration as DurationStr, Evidence},
    hash::Algorithm,
    Hash,
};
use tendermint_rpc as rpc;

use crate::{
    components::{
        clock::Clock,
        io::{AtHeight, Io, IoError},
    },
    errors::Error,
    evidence::EvidenceReporter,
    light_client::LightClient,
    state::State,
    verifier::{
        options::Options,
        types::{Height, LightBlock, PeerId, SignedHeader, Time, TrustThreshold, ValidatorSet},
        ProdVerifier, Verdict, Verifier,
    },
};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Initial {
    pub signed_header: SignedHeader,
    pub next_validator_set: ValidatorSet,
    pub trusting_period: DurationStr,
    pub now: Time,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LightClientTest<LB> {
    pub description: String,
    pub trust_options: TrustOptions,
    pub primary: Provider<LB>,
    pub witnesses: Vec<WitnessProvider<LB>>,
    pub height_to_verify: HeightStr,
    pub now: Time,
    pub expected_output: Option<String>,
    pub expected_num_of_bisections: usize,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WitnessProvider<LB> {
    pub value: Provider<LB>,
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
    pub trust_level: TrustThreshold,
}

#[derive(Clone)]
pub struct MockClock {
    pub now: Time,
}

impl Clock for MockClock {
    fn now(&self) -> Time {
        self.now
    }
}

#[derive(Clone)]
pub struct MockIo {
    light_blocks: HashMap<Height, LightBlock>,
    latest_height: Height,
}

impl MockIo {
    pub fn new(light_blocks: Vec<LightBlock>) -> Self {
        let latest_height = light_blocks.iter().map(|lb| lb.height()).max().unwrap();

        let light_blocks = light_blocks
            .into_iter()
            .map(|lb| (lb.height(), lb))
            .collect();

        Self {
            light_blocks,
            latest_height,
        }
    }
}

impl Io for MockIo {
    fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, IoError> {
        let height = match height {
            AtHeight::Highest => self.latest_height,
            AtHeight::At(height) => height,
        };

        self.light_blocks.get(&height).cloned().ok_or_else(|| {
            IoError::rpc(rpc::Error::response(
                rpc::response_error::ResponseError::new((-32600).into(), None),
            ))
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct MockEvidenceReporter;

#[contract_trait]
impl EvidenceReporter for MockEvidenceReporter {
    fn report(&self, _e: Evidence, _peer: PeerId) -> Result<Hash, IoError> {
        Ok(Hash::from_bytes(Algorithm::Sha256, &[0; 32]).unwrap())
    }
}

impl MockEvidenceReporter {
    pub fn new() -> Self {
        Self
    }
}

pub fn verify_single(
    trusted_block: LightBlock,
    input: LightBlock,
    trust_threshold: TrustThreshold,
    trusting_period: Duration,
    clock_drift: Duration,
    now: Time,
) -> Result<LightBlock, Verdict> {
    trait IntoResult<T, E> {
        fn into_result(self, ok: T) -> Result<T, E>;
    }

    impl<T> IntoResult<T, Verdict> for Verdict {
        fn into_result(self, ok: T) -> Result<T, Verdict> {
            match self {
                Verdict::Success => Ok(ok),
                error => Err(error),
            }
        }
    }

    let verifier = ProdVerifier::default();
    let options = Options {
        trust_threshold,
        trusting_period,
        clock_drift,
    };

    verifier
        .verify(
            input.as_untrusted_state(),
            trusted_block.as_trusted_state(),
            &options,
            now,
        )
        .into_result(input)
}

pub fn verify_bisection(
    untrusted_height: Height,
    light_client: &mut LightClient,
    state: &mut State,
) -> Result<Vec<LightBlock>, Error> {
    light_client
        .verify_to_target(untrusted_height, state)
        .map(|_| state.get_trace(untrusted_height))
}
