use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json;
use std::{fs, path::PathBuf};
use tendermint::block::Header;
use tendermint::lite::{TrustThresholdFraction, TrustedState};
use tendermint::{block::signed_header::SignedHeader, lite, validator::Set, Time};

#[derive(Clone, Debug)]
struct Duration(u64);

#[derive(Deserialize, Clone, Debug)]
struct TestCases {
    batch_name: String,
    test_cases: Vec<TestCase>,
}

#[derive(Deserialize, Clone, Debug)]
struct TestCase {
    description: String,
    initial: Initial,
    input: Vec<LiteBlock>,
    expected_output: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct Initial {
    signed_header: SignedHeader,
    next_validator_set: Set,
    trusting_period: Duration,
    now: Time,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LiteBlock {
    signed_header: SignedHeader,
    validator_set: Set,
    next_validator_set: Set,
}

const TEST_FILES_PATH: &str = "./tests/support/lite/";
fn read_json_fixture(name: &str) -> String {
    fs::read_to_string(PathBuf::from(TEST_FILES_PATH).join(name.to_owned() + ".json")).unwrap()
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

type Trusted = lite::TrustedState<SignedHeader, Header>;

fn run_test_cases(cases: TestCases) {
    for (_, tc) in cases.test_cases.iter().enumerate() {
        let trusted_next_vals = tc.initial.clone().next_validator_set;
        let mut latest_trusted =
            Trusted::new(&tc.initial.signed_header.clone().into(), &trusted_next_vals);
        let expects_err = match &tc.expected_output {
            Some(eo) => eo.eq("error"),
            None => false,
        };

        let trusting_period: std::time::Duration = tc.initial.clone().trusting_period.into();
        let tm_now = tc.initial.now;
        let now = tm_now.to_system_time().unwrap();

        for (i, input) in tc.input.iter().enumerate() {
            println!("i: {}, {}", i, tc.description);
            let untrusted_signed_header = &input.signed_header;
            let untrusted_vals = &input.validator_set;
            let untrusted_next_vals = &input.next_validator_set;
            match lite::verify_single(
                latest_trusted.clone(),
                &untrusted_signed_header.into(),
                &untrusted_vals,
                &untrusted_next_vals,
                TrustThresholdFraction::default(),
                &trusting_period,
                &now,
            ) {
                Ok(new_state) => {
                    let expected_state = TrustedState::new(
                        &untrusted_signed_header.to_owned().into(),
                        untrusted_next_vals,
                    );
                    assert_eq!(new_state, expected_state);
                    assert!(!expects_err);

                    latest_trusted = new_state.clone();
                }
                Err(_) => {
                    assert!(expects_err);
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Duration(
            String::deserialize(deserializer)?
                .parse()
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        ))
    }
}

impl From<Duration> for std::time::Duration {
    fn from(d: Duration) -> std::time::Duration {
        std::time::Duration::from_nanos(d.0)
    }
}
