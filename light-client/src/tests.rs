//! Utilities and datatypes for use in tests.

use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use tendermint::abci::transaction::Hash;
use tendermint::block::Height as HeightStr;
use tendermint::evidence::{Duration as DurationStr, Evidence};
use tendermint::net;

use tendermint_rpc as rpc;

use crate::components::clock::Clock;
use crate::components::io::{AtHeight, Io, IoError};
use crate::components::verifier::{ProdVerifier, Verdict, Verifier};
use crate::errors::Error;
use crate::evidence::EvidenceReporter;
use crate::light_client::{LightClient, Options};
use crate::state::State;
use crate::types::{Height, LightBlock, SignedHeader, Time, TrustThreshold, ValidatorSet};

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
pub struct TestBisection<LB> {
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
    chain_id: String,
    light_blocks: HashMap<Height, LightBlock>,
    latest_height: Height,
}

impl MockIo {
    pub fn new(chain_id: String, light_blocks: Vec<LightBlock>) -> Self {
        let latest_height = light_blocks.iter().map(|lb| lb.height()).max().unwrap();

        let light_blocks = light_blocks
            .into_iter()
            .map(|lb| (lb.height(), lb))
            .collect();

        Self {
            chain_id,
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

        self.light_blocks
            .get(&height)
            .cloned()
            .ok_or_else(|| rpc::Error::new((-32600).into(), None).into())
    }
}

#[derive(Clone, Debug, Default)]
pub struct MockEvidenceReporter;

impl EvidenceReporter for MockEvidenceReporter {
    fn report(&self, _e: Evidence, _peer: net::Address) -> Result<Hash, IoError> {
        Ok(Hash::new([0; 32]))
    }
}

impl MockEvidenceReporter {
    pub fn new() -> Self {
        Self
    }
}

pub fn verify_single(
    trusted_state: Trusted,
    input: LightBlock,
    trust_threshold: TrustThreshold,
    trusting_period: Duration,
    clock_drift: Duration,
    now: Time,
) -> Result<LightBlock, Verdict> {
    let verifier = ProdVerifier::default();

    let trusted_state = LightBlock::new(
        trusted_state.signed_header,
        trusted_state.next_validators.clone(),
        trusted_state.next_validators,
        default_peer_address(),
    );

    let options = Options {
        trust_threshold,
        trusting_period,
        clock_drift,
    };

    let result = verifier.verify(&input, &trusted_state, &options, now);

    match result {
        Verdict::Success => Ok(input),
        error => Err(error),
    }
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

// -----------------------------------------------------------------------------
// Everything below is a temporary workaround for the lack of `provider` field
// in the light blocks serialized in the JSON fixtures.
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AnonLightBlock {
    pub signed_header: SignedHeader,
    #[serde(rename = "validator_set")]
    pub validators: ValidatorSet,
    #[serde(rename = "next_validator_set")]
    pub next_validators: ValidatorSet,
    #[serde(default = "default_peer_address")]
    pub provider: net::Address,
}

pub fn default_peer_address() -> net::Address {
    "tcp://example.com:26656".parse().unwrap()
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

impl From<WitnessProvider<AnonLightBlock>> for WitnessProvider<LightBlock> {
    fn from(p: WitnessProvider<AnonLightBlock>) -> Self {
        Self {
            value: p.value.into(),
        }
    }
}

pub fn peer_address_at(count: usize) -> net::Address {
    let peer_ids: Vec<net::Address> = vec![
        "tcp://example.com:26651".parse().unwrap(),
        "tcp://example.com:26652".parse().unwrap(),
        "tcp://example.com:26653".parse().unwrap(),
        "tcp://example.com:26654".parse().unwrap(),
        "tcp://example.com:26665".parse().unwrap(),
        "tcp://example.com:26657".parse().unwrap(),
    ];

    peer_ids[count].clone()
}

impl From<TestBisection<AnonLightBlock>> for TestBisection<LightBlock> {
    fn from(tb: TestBisection<AnonLightBlock>) -> Self {
        let mut witnesses: Vec<WitnessProvider<LightBlock>> =
            tb.witnesses.into_iter().map(Into::into).collect();

        for (count, provider) in witnesses.iter_mut().enumerate() {
            for lb in provider.value.lite_blocks.iter_mut() {
                lb.provider = peer_address_at(count);
            }
        }

        Self {
            description: tb.description,
            trust_options: tb.trust_options,
            primary: tb.primary.into(),
            witnesses,
            height_to_verify: tb.height_to_verify,
            now: tb.now,
            expected_output: tb.expected_output,
            expected_num_of_bisections: tb.expected_num_of_bisections,
        }
    }
}
