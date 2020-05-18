use light_spike::prelude::*;
use light_spike::tests::{Trusted, *};

use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;

use tendermint::rpc;

const PEER_ID: &str = "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE";

// Link to the commit that generated below JSON test files:
// https://github.com/Shivani912/tendermint/commit/e02f8fd54a278f0192353e54b84a027c8fe31c1e
const TEST_FILES_PATH: &str = "./tests/support/";

fn read_json_fixture(name: &str) -> String {
    fs::read_to_string(PathBuf::from(TEST_FILES_PATH).join(name.to_owned() + ".json")).unwrap()
}

fn verify_single(
    trusted_state: Trusted,
    input: LightBlock,
    trust_threshold: TrustThreshold,
    trusting_period: Duration,
    now: SystemTime,
) -> Result<LightBlock, Verdict> {
    let verifier = ProdVerifier::new(
        ProdPredicates,
        ProdVotingPowerCalculator,
        ProdCommitValidator,
        ProdHeaderHasher,
    );

    let trusted_state = LightBlock::new(
        trusted_state.signed_header,
        trusted_state.next_validators.clone(),
        trusted_state.next_validators,
        PEER_ID.parse().unwrap(),
    );

    let options = VerificationOptions {
        trust_threshold,
        trusting_period,
        now: now.into(),
    };

    let result = verifier
        .validate_light_block(&input, &trusted_state, &options)
        .and_then(|| verifier.verify_overlap(&input, &trusted_state, &options))
        .and_then(|| verifier.has_sufficient_voting_power(&input, &options));

    match result {
        Verdict::Success => Ok(input),
        error => Err(error),
    }
}

fn run_test_cases(cases: TestCases) {
    for tc in cases.test_cases.iter() {
        let mut latest_trusted = Trusted::new(
            tc.initial.signed_header.clone(),
            tc.initial.next_validator_set.clone(),
        );

        let expects_err = match &tc.expected_output {
            Some(eo) => eo.eq("error"),
            None => false,
        };

        let trusting_period: Duration = tc.initial.trusting_period.into();
        let tm_now = tc.initial.now;
        let now = tm_now.to_system_time().unwrap();

        for (i, input) in tc.input.iter().enumerate() {
            println!("i: {}, {}", i, tc.description);

            match verify_single(
                latest_trusted.clone(),
                input.clone(),
                TrustThreshold::default(),
                trusting_period.into(),
                now,
            ) {
                Ok(new_state) => {
                    let expected_state = input;

                    assert_eq!(new_state.height(), expected_state.height());
                    assert_eq!(&new_state, expected_state);
                    assert!(!expects_err);

                    latest_trusted =
                        Trusted::new(new_state.signed_header, new_state.next_validators);
                }
                Err(_) => {
                    assert!(expects_err);
                }
            }
        }
    }
}

#[test]
fn val_set_tests_verify() {
    let cases: TestCases =
        serde_json::from_str(&read_json_fixture("single_step_sequential/val_set_tests")).unwrap();
    run_test_cases(cases);
}

#[test]
fn commit_tests_verify() {
    let cases: TestCases =
        serde_json::from_str(&read_json_fixture("single_step_sequential/commit_tests")).unwrap();
    run_test_cases(cases);
}

#[test]
fn header_tests_verify() {
    let cases: TestCases =
        serde_json::from_str(&read_json_fixture("single_step_sequential/header_tests")).unwrap();
    run_test_cases(cases);
}

#[test]
fn single_skip_val_set_tests_verify() {
    let cases: TestCases =
        serde_json::from_str(&read_json_fixture("single_step_skipping/val_set_tests")).unwrap();
    run_test_cases(cases);
}

#[test]
fn single_skip_commit_tests_verify() {
    let cases: TestCases =
        serde_json::from_str(&read_json_fixture("single_step_skipping/commit_tests")).unwrap();
    run_test_cases(cases);
}

#[derive(Clone)]
struct MockIo {
    chain_id: String,
    light_blocks: HashMap<Height, LightBlock>,
}

impl MockIo {
    fn new(chain_id: String, light_blocks: Vec<LightBlock>) -> Self {
        let light_blocks = light_blocks
            .into_iter()
            .map(|lb| (lb.height(), lb))
            .collect();

        Self {
            chain_id,
            light_blocks,
        }
    }
}

impl Io for MockIo {
    fn fetch_light_block(&mut self, _peer: PeerId, height: Height) -> Result<LightBlock, IoError> {
        self.light_blocks
            .get(&height)
            .cloned()
            .ok_or(rpc::Error::new((-32600).into(), None).into())
    }
}

struct MockClock {
    now: Time,
}

impl Clock for MockClock {
    fn now(&self) -> Time {
        self.now
    }
}

fn verify_bisection(
    untrusted_height: Height,
    demuxer: &mut LightClient,
) -> Result<Vec<LightBlock>, Error> {
    demuxer
        .verify_to_target(untrusted_height)
        .map(|()| demuxer.get_trace(untrusted_height))
}

fn run_bisection_test(case: TestBisection) {
    println!("{}", case.description);

    let primary: PeerId = PEER_ID.parse().unwrap();

    let untrusted_height = case.height_to_verify.try_into().unwrap();
    let trust_threshold = case.trust_options.trust_level;
    let trusting_period = case.trust_options.period;
    let now = case.now;

    let clock = MockClock { now };
    let scheduler = light_spike::components::scheduler::schedule;
    let fork_detector = RealForkDetector::new(ProdHeaderHasher);

    let options = VerificationOptions {
        trust_threshold,
        trusting_period: trusting_period.into(),
        now,
    };

    let expects_err = match &case.expected_output {
        Some(eo) => eo.eq("error"),
        None => false,
    };

    let expected_num_of_bisections = case.expected_num_of_bisections;

    let provider = case.primary;
    let mut io = MockIo::new(provider.chain_id, provider.lite_blocks);

    let trusted_height = case.trust_options.height.try_into().unwrap();
    let trusted_state = io
        .fetch_light_block(primary.clone(), trusted_height)
        .expect("could not 'request' light block");

    let mut light_store = MemoryStore::new();
    light_store.insert(trusted_state, VerifiedStatus::Verified);

    let state = State {
        peers: Peers {
            primary: primary.clone(),
            witnesses: vec![],
        },
        light_store: Box::new(light_store),
        verification_trace: HashMap::new(),
    };

    let verifier = ProdVerifier::new(
        ProdPredicates,
        ProdVotingPowerCalculator,
        ProdCommitValidator,
        ProdHeaderHasher,
    );

    let mut demuxer = LightClient::new(
        state,
        options,
        clock,
        scheduler,
        verifier,
        fork_detector,
        io.clone(),
    );

    match verify_bisection(untrusted_height, &mut demuxer) {
        Ok(new_states) => {
            let untrusted_light_block = io
                .fetch_light_block(primary.clone(), untrusted_height)
                .expect("header at untrusted height not found");

            assert_eq!(new_states.len(), expected_num_of_bisections);

            let expected_state = untrusted_light_block;
            assert_eq!(new_states[0].height(), expected_state.height());
            assert_eq!(new_states[0], expected_state);
            assert!(!expects_err);
        }
        Err(e) => {
            if !expects_err {
                dbg!(e);
            }
            assert!(expects_err);
        }
    }
}

#[test]
fn bisection_happy_path() {
    let case: TestBisection =
        serde_json::from_str(&read_json_fixture("many_header_bisection/happy_path")).unwrap();
    run_bisection_test(case);
}

#[test]
fn bisection_header_out_of_trusting_period() {
    let case: TestBisection = serde_json::from_str(&read_json_fixture(
        "many_header_bisection/header_out_of_trusting_period",
    ))
    .unwrap();
    run_bisection_test(case);
}

#[test]
fn bisection_invalid_validator_set() {
    let case: TestBisection = serde_json::from_str(&read_json_fixture(
        "many_header_bisection/invalid_validator_set",
    ))
    .unwrap();
    run_bisection_test(case);
}

#[test]
fn bisection_not_enough_commits() {
    let case: TestBisection = serde_json::from_str(&read_json_fixture(
        "many_header_bisection/not_enough_commits",
    ))
    .unwrap();
    run_bisection_test(case);
}

#[test]
fn bisection_worst_case() {
    let case: TestBisection =
        serde_json::from_str(&read_json_fixture("many_header_bisection/worst_case")).unwrap();
    run_bisection_test(case);
}
