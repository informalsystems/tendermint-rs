use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use tendermint_light_client::{
    components::{
        io::{AtHeight, Io},
        scheduler,
        verifier::{ProdVerifier, Verdict, Verifier},
    },
    errors::{Error, ErrorKind},
    light_client::{LightClient, Options},
    state::State,
    store::{memory::MemoryStore, LightStore},
    tests::{Trusted, *},
    types::{Height, LightBlock, Status, TMLightBlock, TrustThreshold},
};

// Link to the commit that generated below JSON test files:
// https://github.com/Shivani912/tendermint/commit/e02f8fd54a278f0192353e54b84a027c8fe31c1e
const TEST_FILES_PATH: &str = "./tests/support/";

fn read_json_fixture(file: impl AsRef<Path>) -> String {
    fs::read_to_string(file).unwrap()
}

fn verify_single(
    trusted_state: Trusted,
    input: TMLightBlock,
    trust_threshold: TrustThreshold,
    trusting_period: Duration,
    clock_drift: Duration,
    now: SystemTime,
) -> Result<TMLightBlock, Verdict> {
    let verifier = ProdVerifier::default();

    let trusted_state = TMLightBlock::new(
        trusted_state.signed_header,
        trusted_state.next_validators.clone(),
        trusted_state.next_validators,
        default_peer_id(),
    );

    let options = Options {
        trust_threshold,
        trusting_period,
        clock_drift,
    };

    let result = verifier.verify(&input, &trusted_state, &options, now.into());

    match result {
        Verdict::Success => Ok(input),
        error => Err(error),
    }
}

fn run_test_case(tc: TestCase<TMLightBlock>) {
    let mut latest_trusted = Trusted::new(
        tc.initial.signed_header.clone(),
        tc.initial.next_validator_set.clone(),
    );

    let expects_err = match &tc.expected_output {
        Some(eo) => eo.eq("error"),
        None => false,
    };

    // In Go, default is 10 sec.
    // Once we switch to the proposer based timestamps, it will probably be a consensus parameter
    let clock_drift = Duration::from_secs(10);

    let trusting_period: Duration = tc.initial.trusting_period.into();
    let tm_now = tc.initial.now;
    let now = tm_now.to_system_time().unwrap();

    for (i, input) in tc.input.iter().enumerate() {
        println!("  - {}: {}", i, tc.description);

        match verify_single(
            latest_trusted.clone(),
            input.clone(),
            TrustThreshold::default(),
            trusting_period,
            clock_drift,
            now,
        ) {
            Ok(new_state) => {
                let expected_state = input;

                assert_eq!(new_state.height(), expected_state.height());
                assert_eq!(&new_state, expected_state);
                assert!(!expects_err);

                latest_trusted = Trusted::new(new_state.signed_header, new_state.next_validators);
            }
            Err(e) => {
                if !expects_err {
                    dbg!(e);
                }
                assert!(expects_err);
            }
        }
    }
}

fn verify_bisection(
    untrusted_height: Height,
    light_client: &mut LightClient<TMLightBlock>,
    state: &mut State<TMLightBlock>,
) -> Result<Vec<TMLightBlock>, Error> {
    light_client
        .verify_to_target(untrusted_height, state)
        .map(|_| state.get_trace(untrusted_height))
}

struct BisectionTestResult {
    untrusted_light_block: TMLightBlock,
    new_states: Result<Vec<TMLightBlock>, Error>,
}

fn run_bisection_test(tc: TestBisection<TMLightBlock>) -> BisectionTestResult {
    println!("  - {}", tc.description);

    let primary = default_peer_id();
    let untrusted_height = tc.height_to_verify.try_into().unwrap();
    let trust_threshold = tc.trust_options.trust_level;
    let trusting_period = tc.trust_options.period;
    let now = tc.now;

    // In Go, default is 10 sec.
    // Once we switch to the proposer based timestamps, it will probably be a consensus parameter
    let clock_drift = Duration::from_secs(10);

    let clock = MockClock { now };

    let options = Options {
        trust_threshold,
        trusting_period: trusting_period.into(),
        clock_drift,
    };

    let provider = tc.primary;
    let io = MockIo::new(provider.chain_id, provider.lite_blocks);

    let trusted_height = tc.trust_options.height.try_into().unwrap();
    let trusted_state = io
        .fetch_light_block(primary, AtHeight::At(trusted_height))
        .expect("could not 'request' light block");

    let mut light_store = MemoryStore::new();
    light_store.insert(trusted_state, Status::Trusted);

    let mut state = State {
        light_store: Box::new(light_store),
        verification_trace: HashMap::new(),
    };

    let verifier = ProdVerifier::default();

    let mut light_client = LightClient::new(
        primary,
        options,
        clock,
        scheduler::basic_bisecting_schedule,
        verifier,
        io.clone(),
    );

    let result = verify_bisection(untrusted_height, &mut light_client, &mut state);

    let untrusted_light_block = io
        .fetch_light_block(primary, AtHeight::At(untrusted_height))
        .expect("header at untrusted height not found");

    BisectionTestResult {
        untrusted_light_block,
        new_states: result,
    }
}

fn run_single_step_tests(dir: &str) {
    let paths = fs::read_dir(PathBuf::from(TEST_FILES_PATH).join(dir)).unwrap();

    for file_path in paths {
        let dir_entry = file_path.unwrap();
        let fp_str = format!("{}", dir_entry.path().display());

        println!(
            "Running light client against 'single-step' test-file: {}",
            fp_str
        );

        let case = read_test_case(&fp_str);
        run_test_case(case);
    }
}

fn foreach_bisection_test(dir: &str, f: impl Fn(String, TestBisection<TMLightBlock>) -> ()) {
    let paths = fs::read_dir(PathBuf::from(TEST_FILES_PATH).join(dir)).unwrap();

    for file_path in paths {
        let dir_entry = file_path.unwrap();
        let fp_str = format!("{}", dir_entry.path().display());
        let tc = read_bisection_test_case(&fp_str);
        f(fp_str, tc);
    }
}

fn run_bisection_tests(dir: &str) {
    foreach_bisection_test(dir, |file, tc| {
        println!("Running light client against bisection test-file: {}", file);

        let expect_error = match &tc.expected_output {
            Some(eo) => eo.eq("error"),
            None => false,
        };

        let test_result = run_bisection_test(tc);
        let expected_state = test_result.untrusted_light_block;

        match test_result.new_states {
            Ok(new_states) => {
                assert_eq!(new_states[0].height(), expected_state.height());
                assert_eq!(new_states[0], expected_state);
                assert!(!expect_error);
            }
            Err(e) => {
                if !expect_error {
                    dbg!(e);
                }
                assert!(expect_error);
            }
        }
    });
}

/// Test that the light client fails with `ErrorKind::TargetLowerThanTrustedState`
/// when the target height is lower than the last trusted state height.
///
/// To do this, we override increment the trusted height by 1
/// and set the target height to `trusted_height - 1`, then run
/// the bisection test as normal. We then assert that we get the expected error.
fn run_bisection_lower_tests(dir: &str) {
    foreach_bisection_test(dir, |file, mut tc| {
        let mut trusted_height: Height = tc.trust_options.height.into();

        if trusted_height <= 1 {
            tc.trust_options.height = (trusted_height + 1).into();
            trusted_height += 1;
        }

        println!(
            "Running light client against bisection test file with target height too low: {}",
            file
        );

        tc.height_to_verify = (trusted_height - 1).into();

        let test_result = run_bisection_test(tc);
        match test_result.new_states {
            Ok(_) => {
                panic!("test unexpectedly succeeded, expected TargetLowerThanTrustedState error");
            }
            Err(e) => match e.kind() {
                ErrorKind::TargetLowerThanTrustedState { .. } => (),
                kind => panic!(
                    "unexpected error, expected: TargetLowerThanTrustedState, got: {}",
                    kind
                ),
            },
        }
    });
}

fn read_test_case(file_path: &str) -> TestCase<TMLightBlock> {
    let tc: TestCase<AnonLightBlock> =
        serde_json::from_str(read_json_fixture(file_path).as_str()).unwrap();
    tc.into()
}

fn read_bisection_test_case(file_path: &str) -> TestBisection<TMLightBlock> {
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
fn bisection_lower() {
    let dir = "bisection/single_peer";
    run_bisection_lower_tests(dir);
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
