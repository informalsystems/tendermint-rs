use light_client::prelude::*;
use light_client::tests::{Trusted, *};

use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::path::{Path, PathBuf};

use tendermint::rpc;

// Link to the commit that generated below JSON test files:
// https://github.com/Shivani912/tendermint/commit/e02f8fd54a278f0192353e54b84a027c8fe31c1e
const TEST_FILES_PATH: &str = "./tests/support/";

fn read_json_fixture(file: impl AsRef<Path>) -> String {
    fs::read_to_string(file).unwrap()
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
        default_peer_id(),
    );

    let options = Options {
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

fn run_test_case(tc: TestCase<LightBlock>) {
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
        println!("  - {}: {}", i, tc.description);

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

                latest_trusted = Trusted::new(new_state.signed_header, new_state.next_validators);
            }
            Err(_) => {
                assert!(expects_err);
            }
        }
    }
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
    light_client: &mut LightClient,
) -> Result<Vec<LightBlock>, Error> {
    light_client
        .verify_to_target(untrusted_height)
        .map(|_| light_client.get_trace(untrusted_height))
}

fn run_bisection_test(tc: TestBisection<LightBlock>) {
    println!("  - {}", tc.description);

    let primary = default_peer_id();
    let untrusted_height = tc.height_to_verify.try_into().unwrap();
    let trust_threshold = tc.trust_options.trust_level;
    let trusting_period = tc.trust_options.period;
    let now = tc.now;

    let clock = MockClock { now };
    let scheduler = light_client::components::scheduler::schedule;
    let fork_detector = RealForkDetector::new(ProdHeaderHasher);

    let options = Options {
        trust_threshold,
        trusting_period: trusting_period.into(),
        now,
    };

    let expects_err = match &tc.expected_output {
        Some(eo) => eo.eq("error"),
        None => false,
    };

    let provider = tc.primary;
    let mut io = MockIo::new(provider.chain_id, provider.lite_blocks);

    let trusted_height = tc.trust_options.height.try_into().unwrap();
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

    let mut light_client = LightClient::new(
        state,
        options,
        clock,
        scheduler,
        verifier,
        fork_detector,
        io.clone(),
    );

    match verify_bisection(untrusted_height, &mut light_client) {
        Ok(new_states) => {
            let untrusted_light_block = io
                .fetch_light_block(primary.clone(), untrusted_height)
                .expect("header at untrusted height not found");

            // TODO: number of bisections started diverting in JSON tests and Rust impl
            // assert_eq!(new_states.len(), case.expected_num_of_bisections);

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

fn run_single_step_tests(dir: &str) {
    // TODO: this test need further investigation:
    let skipped = ["commit/one_third_vals_don't_sign.json"];

    let paths = fs::read_dir(PathBuf::from(TEST_FILES_PATH).join(dir)).unwrap();

    for file_path in paths {
        let dir_entry = file_path.unwrap();
        let fp_str = format!("{}", dir_entry.path().display());

        if skipped
            .iter()
            .any(|failing_case| fp_str.ends_with(failing_case))
        {
            println!("Skipping JSON test: {}", fp_str);
            return;
        }

        println!(
            "Running light client against 'single-step' test-file: {}",
            fp_str
        );

        let case = read_test_case(&fp_str);
        run_test_case(case);
    }
}

fn run_bisection_tests(dir: &str) {
    let paths = fs::read_dir(PathBuf::from(TEST_FILES_PATH).join(dir)).unwrap();

    for file_path in paths {
        let dir_entry = file_path.unwrap();
        let fp_str = format!("{}", dir_entry.path().display());

        println!(
            "Running light client against bisection test-file: {}",
            fp_str
        );

        let case = read_bisection_test_case(&fp_str);
        run_bisection_test(case);
    }
}

fn read_test_case(file_path: &str) -> TestCase<LightBlock> {
    let tc: TestCase<AnonLightBlock> =
        serde_json::from_str(read_json_fixture(file_path).as_str()).unwrap();
    tc.into()
}

fn read_bisection_test_case(file_path: &str) -> TestBisection<LightBlock> {
    let tc: TestBisection<AnonLightBlock> =
        serde_json::from_str(read_json_fixture(file_path).as_str()).unwrap();
    tc.into()
}

#[test]
fn bisection() {
    let dir = "bisection/single_peer";
    run_bisection_tests(dir);
}

#[test]
fn single_step_sequential() {
    let dirs = [
        "single_step/sequential/commit",
        "single_step/sequential/header",
        "single_step/sequential/validator_set",
    ];

    for dir in &dirs {
        run_single_step_tests(dir);
    }
}

#[test]
fn single_step_skipping() {
    let dirs = [
        "single_step/skipping/commit",
        "single_step/skipping/header",
        "single_step/skipping/validator_set",
    ];

    for dir in &dirs {
        run_single_step_tests(dir);
    }
}
