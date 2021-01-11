//! Utilities and datatypes for use in tests.

use crate::types::{Height, LightBlock, PeerId, SignedHeader, Time, TrustThreshold, ValidatorSet};

use serde::{Deserialize, Serialize};
use tendermint::abci::transaction::Hash;
use tendermint_rpc as rpc;

use crate::components::clock::Clock;
use crate::components::io::{AtHeight, Io, IoError};
use crate::components::verifier::{ProdVerifier, Verdict, Verifier};
use crate::errors::Error;
use crate::evidence::EvidenceReporter;
use crate::light_client::{LightClient, Options};
use crate::state::State;
use contracts::contract_trait;
use std::collections::HashMap;
use std::time::Duration;
use tendermint::block::Height as HeightStr;
use tendermint::evidence::{Duration as DurationStr, Evidence};

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
    pub witnesses: Vec<Provider<LB>>,
    pub height_to_verify: HeightStr,
    pub now: Time,
    pub expected_output: BisectionVerdict,
    pub expected_num_of_bisections: usize,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum BisectionVerdict {
    /// verified successfully
    #[serde(rename = "SUCCESS")]
    Success,

    /// No primary
    #[serde(rename = "NO_PRIMARY")]
    NoPrimary,

    /// No witnesses
    #[serde(rename = "NO_WITNESSES")]
    NoWitnesses,

    /// No witness left
    #[serde(rename = "NO_WITNESS_LEFT")]
    NoWitnessLeft,

    /// A fork has been detected between some peers
    #[serde(rename = "FORK_DETECTED")]
    ForkDetected,

    /// No initial trusted state
    #[serde(rename = "NO_INITIAL_TRUSTED_STATE")]
    NoInitialTrustedState,

    /// No trusted state
    #[serde(rename = "NO_TRUSTED_STATE")]
    NoTrustedState,

    /// Target height for the light client lower than latest trusted state height
    #[serde(rename = "TARGET_LOWER_THAN_TRUSTED_STATE")]
    TargetLowerThanTrustedState,

    /// The trusted state is outside of the trusting period
    #[serde(rename = "TRUSTED_STATE_OUTSIDE_TRUSTING_PERIOD")]
    TrustedStateOutsideTrustingPeriod,

    /// Bisection failed when reached trusted state
    #[serde(rename = "BISECTION_FAILED")]
    BisectionFailed,

    /// Verification failed for a light block
    #[serde(rename = "INVALID_LIGHT_BLOCK")]
    InvalidLightBlock,
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

#[contract_trait]
impl EvidenceReporter for MockEvidenceReporter {
    fn report(&self, _e: Evidence, _peer: PeerId) -> Result<Hash, IoError> {
        Ok(Hash::new([0; 32]))
    }
}

impl MockEvidenceReporter {
    pub fn new() -> Self {
        Self
    }
}

pub fn verify_single(
    trusted_state: LightBlock,
    input: LightBlock,
    trust_threshold: TrustThreshold,
    trusting_period: Duration,
    clock_drift: Duration,
    now: Time,
) -> Result<LightBlock, Verdict> {
    let verifier = ProdVerifier::default();

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
