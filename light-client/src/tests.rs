//! Utilities and datatypes for use in tests.

use crate::types::{
    Height, LightBlock, PeerId, TMLightBlock, TMSignedHeader, TMValidatorSet, Time, TrustThreshold,
};

use contracts::contract_trait;
use serde::Deserialize;
use std::collections::HashMap;
use tendermint_rpc as rpc;

use crate::components::clock::Clock;
use crate::components::io::{AtHeight, Io, IoError};
use crate::evidence::EvidenceReporter;

use tendermint::abci::transaction::Hash;
use tendermint::block::Height as HeightStr;
use tendermint::evidence::{ConflictingHeadersEvidence, Duration as DurationStr, Evidence};

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
    pub signed_header: TMSignedHeader,
    pub next_validator_set: TMValidatorSet,
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
    pub signed_header: TMSignedHeader,
    pub next_validators: TMValidatorSet,
}

impl Trusted {
    pub fn new(signed_header: TMSignedHeader, next_validators: TMValidatorSet) -> Self {
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
    light_blocks: HashMap<Height, TMLightBlock>,
    latest_height: Height,
}

impl MockIo {
    pub fn new(chain_id: String, light_blocks: Vec<TMLightBlock>) -> Self {
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

#[contract_trait]
impl Io<TMLightBlock> for MockIo {
    fn fetch_light_block(&self, _peer: PeerId, height: AtHeight) -> Result<TMLightBlock, IoError> {
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

#[contract_trait]
impl EvidenceReporter<TMLightBlock> for MockEvidenceReporter {
    fn report(&self, _e: Evidence, _peer: PeerId) -> Result<Hash, IoError> {
        Ok(Hash::new([0; 32]))
    }

    fn build_conflicting_headers_evidence(
        &self,
        h1: TMSignedHeader,
        h2: TMSignedHeader,
    ) -> Evidence {
        Evidence::ConflictingHeaders(Box::new(ConflictingHeadersEvidence::new(h1, h2)))
    }
}

impl MockEvidenceReporter {
    pub fn new() -> Self {
        Self
    }
}

// -----------------------------------------------------------------------------
// Everything below is a temporary workaround for the lack of `provider` field
// in the light blocks serialized in the JSON fixtures.
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct AnonLightBlock {
    pub signed_header: TMSignedHeader,
    #[serde(rename = "validator_set")]
    pub validators: TMValidatorSet,
    #[serde(rename = "next_validator_set")]
    pub next_validators: TMValidatorSet,
    #[serde(default = "default_peer_id")]
    pub provider: PeerId,
}

pub fn default_peer_id() -> PeerId {
    "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap()
}

impl From<AnonLightBlock> for TMLightBlock {
    fn from(alb: AnonLightBlock) -> Self {
        Self {
            signed_header: alb.signed_header,
            validators: alb.validators,
            next_validators: alb.next_validators,
            provider: alb.provider,
        }
    }
}

impl From<TestCase<AnonLightBlock>> for TestCase<TMLightBlock> {
    fn from(tc: TestCase<AnonLightBlock>) -> Self {
        Self {
            description: tc.description,
            initial: tc.initial,
            input: tc.input.into_iter().map(Into::into).collect(),
            expected_output: tc.expected_output,
        }
    }
}

impl From<TestCases<AnonLightBlock>> for TestCases<TMLightBlock> {
    fn from(tc: TestCases<AnonLightBlock>) -> Self {
        Self {
            batch_name: tc.batch_name,
            test_cases: tc.test_cases.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Provider<AnonLightBlock>> for Provider<TMLightBlock> {
    fn from(p: Provider<AnonLightBlock>) -> Self {
        Self {
            chain_id: p.chain_id,
            lite_blocks: p.lite_blocks.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<WitnessProvider<AnonLightBlock>> for WitnessProvider<TMLightBlock> {
    fn from(p: WitnessProvider<AnonLightBlock>) -> Self {
        Self {
            value: p.value.into(),
        }
    }
}

pub fn peer_id_at(count: usize) -> PeerId {
    let peer_ids: Vec<PeerId> = vec![
        "ADBEEFC0FFEEFACADEBADFADADE0BEFEEDC0C0AD".parse().unwrap(),
        "BADFADBEEFC0FFEEFACADEAD0BEFEEDC0C0ADEAD".parse().unwrap(),
        "CADEEDC0C0ADEADBEEFBADFADAD0BEFEC0FFEEFA".parse().unwrap(),
        "D0BEFEEDC0C0ADEABADFADADBEEFC0FFEEFACADE".parse().unwrap(),
        "EEFC0FFEEFACADEBADFADAD0BEFEEDC0C0ADEADB".parse().unwrap(),
        "FC0FFEEFABAC0C0ADEADDFBEEADAD0BEFEEDCADE".parse().unwrap(),
    ];
    peer_ids[count]
}

impl From<TestBisection<AnonLightBlock>> for TestBisection<TMLightBlock> {
    fn from(tb: TestBisection<AnonLightBlock>) -> Self {
        let mut witnesses: Vec<WitnessProvider<TMLightBlock>> =
            tb.witnesses.into_iter().map(Into::into).collect();

        for (count, provider) in witnesses.iter_mut().enumerate() {
            for lb in provider.value.lite_blocks.iter_mut() {
                lb.provider = peer_id_at(count);
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
