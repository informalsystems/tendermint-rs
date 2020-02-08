use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json;
use std::collections::HashMap;
use std::convert::TryInto;
use std::{fs, path::PathBuf};
use tendermint::block::{Header, Height};
use tendermint::lite::{Error, Requester, TrustThresholdFraction, TrustedState};
use tendermint::{block::signed_header::SignedHeader, lite, validator::Set, Hash, Time};

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

#[derive(Deserialize, Clone, Debug)]
struct TestBisection {
    description: String,
    trust_options: TrustOptions,
    primary: Provider,
    height_to_verify: Height,
    trust_level: TrustThresholdFraction,
    now: Time,
    expected_output: String,
}

#[derive(Deserialize, Clone, Debug)]
struct Provider {
    chain_id: String,
    lite_blocks: Vec<LiteBlock>,
}

#[derive(Deserialize, Clone, Debug)]
struct TrustOptions {
    period: Duration,
    height: Height,
    hash: Hash,
}

#[derive(Deserialize, Clone, Debug)]
struct MockRequester {
    chain_id: String,
    signed_headers: HashMap<u64, SignedHeader>,
    validators: HashMap<u64, Set>,
}

type LightSignedHeader = lite::types::SignedHeader<SignedHeader, Header>;

impl Requester<SignedHeader, Header> for MockRequester {
    fn signed_header(&self, h: u64) -> Result<LightSignedHeader, Error> {
        println!("requested signed header for height:{:?}", h);
        if let Some(sh) = self.signed_headers.get(&h) {
            return Ok(sh.into());
        }
        println!("couldn't get sh for: {}", &h);
        Err(Error::RequestFailed)
    }

    fn validator_set(&self, h: u64) -> Result<Set, Error> {
        println!("requested validators for height:{:?}", h);
        if let Some(vs) = self.validators.get(&h) {
            return Ok(vs.to_owned());
        }
        println!("couldn't get vals for: {}", &h);
        Err(Error::RequestFailed)
    }
}

impl MockRequester {
    fn new(chain_id: String, lite_blocks: Vec<LiteBlock>) -> Self {
        let mut sh_map: HashMap<u64, SignedHeader> = HashMap::new();
        let mut val_map: HashMap<u64, Set> = HashMap::new();
        for lite_block in lite_blocks {
            let height = lite_block.signed_header.header.height;
            sh_map.insert(height.into(), lite_block.signed_header);
            val_map.insert(height.into(), lite_block.validator_set);
        }
        Self {
            chain_id,
            signed_headers: sh_map,
            validators: val_map,
        }
    }
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

#[test]
fn bisection_simple() {
    let case: TestBisection =
        serde_json::from_str(&read_json_fixture("many_header_bisection/happy_path")).unwrap();
    run_bisection_test(case);
}

fn run_bisection_test(case: TestBisection) {
    println!("{}", case.description);

    let untrusted_height = case.height_to_verify.try_into().unwrap();
    let trust_threshold = case.trust_level;
    let trusting_period = case.trust_options.period;
    let now = case.now;

    let provider = case.primary;
    let req = MockRequester::new(provider.chain_id, provider.lite_blocks);

    let expected_output = case.expected_output;

    let trusted_height = case.trust_options.height.try_into().unwrap();
    let trusted_header = &req
        .signed_header(trusted_height)
        .expect("could not 'request' signed header");
    let trusted_vals = &req
        .validator_set(trusted_height + 1)
        .expect("could not 'request' validator set");

    let trusted_state = TrustedState::new(trusted_header, trusted_vals);

    let output: String;

    match lite::verify_bisection(
        trusted_state,
        untrusted_height,
        trust_threshold,
        &trusting_period.into(),
        &now.into(),
        &req,
    ) {
        Ok(_) => {
            // TODO: should we make some assertions on the returned new_states?
            output = "no error".to_string();
        }
        Err(_) => {
            output = "error".to_string();
        }
    }
    assert_eq!(output, expected_output);
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
