use light_spike::prelude::*;

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use tendermint::evidence::Duration as DurationStr;

#[derive(Deserialize, Clone, Debug)]
struct TestCases {
    batch_name: String,
    test_cases: Vec<TestCase>,
}

#[derive(Deserialize, Clone, Debug)]
struct TestCase {
    description: String,
    initial: Initial,
    input: Vec<LightBlock>,
    expected_output: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct Initial {
    signed_header: SignedHeader,
    next_validator_set: ValidatorSet,
    trusting_period: DurationStr,
    now: Time,
}

#[derive(Deserialize, Clone, Debug)]
struct TestBisection {
    description: String,
    trust_options: TrustOptions,
    primary: Provider,
    height_to_verify: Height,
    now: Time,
    expected_output: Option<String>,
    expected_num_of_bisections: i32,
}

#[derive(Deserialize, Clone, Debug)]
struct Provider {
    chain_id: String,
    lite_blocks: Vec<LightBlock>,
}

#[derive(Deserialize, Clone, Debug)]
struct TrustOptions {
    period: Duration,
    height: Height,
    hash: Hash,
    trust_level: TrustThreshold,
}

#[derive(Deserialize, Clone, Debug)]
struct Trusted {
    signed_header: SignedHeader,
    next_validators: ValidatorSet,
}

impl Trusted {
    fn new(signed_header: SignedHeader, next_validators: ValidatorSet) -> Self {
        Self {
            signed_header,
            next_validators,
        }
    }
}

// Link to the commit that generated below JSON test files:
// https://github.com/Shivani912/tendermint/commit/e02f8fd54a278f0192353e54b84a027c8fe31c1e
const TEST_FILES_PATH: &str = "./tests/support/";

fn read_json_fixture(name: &str) -> String {
    fs::read_to_string(PathBuf::from(TEST_FILES_PATH).join(name.to_owned() + ".json")).unwrap()
}

fn verify_single(
    trusted_state: Trusted,
    input: &LightBlock,
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

    let provider = "tcp://localhost:1337".parse().unwrap();
    let trusted_state = LightBlock::new(
        trusted_state.signed_header,
        trusted_state.next_validators.clone(),
        trusted_state.next_validators,
        provider,
    );

    let options = VerificationOptions {
        trust_threshold,
        trusting_period,
        now: now.into(),
    };

    verifier
        .validate_light_block(input, &trusted_state, &options)
        .ok()?;

    verifier
        .verify_overlap(input, &trusted_state, &options)
        .ok()?;

    verifier.has_sufficient_voting_power(input, &options).ok()?;

    Ok(input.clone())
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
                input,
                TrustThreshold::default(),
                trusting_period.into(),
                now,
            ) {
                Ok(new_state) => {
                    let expected_state = input;

                    assert_eq!(&new_state, expected_state);
                    assert!(!expects_err);

                    latest_trusted =
                        Trusted::new(new_state.signed_header, new_state.next_validators);
                }
                Err(verdict) => {
                    if !expects_err {
                        dbg!(verdict);
                    }
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
