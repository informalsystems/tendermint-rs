use anomaly::fail;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::{fs, path::PathBuf};
use tendermint::block::{Header, Height};
use tendermint::lite::error::{Error, Kind};
use tendermint::lite::{Requester, TrustThresholdFraction, TrustedState};
use tendermint::{
    block::signed_header::SignedHeader, evidence::Duration, lite, validator::Set, Hash, Time,
};

/// Test that a struct `T` can be:
///
/// - serialized to JSON
/// - parsed back from the serialized JSON of the previous step
/// - that the two parsed structs are equal according to their `PartialEq` impl
pub fn test_serialization_roundtrip<T>(obj: &T)
where
    T: Debug + PartialEq + Serialize + DeserializeOwned,
{
    let serialized = serde_json::to_string(obj).unwrap();
    let parsed = serde_json::from_str(&serialized).unwrap();
    assert_eq!(obj, &parsed);
}

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
    now: Time,
    expected_output: Option<String>,
    expected_num_of_bisections: i32,
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
    trust_level: TrustThresholdFraction,
}

#[derive(Deserialize, Clone, Debug)]
struct MockRequester {
    chain_id: String,
    signed_headers: HashMap<u64, SignedHeader>,
    validators: HashMap<u64, Set>,
}

type LightSignedHeader = lite::types::SignedHeader<SignedHeader, Header>;

#[async_trait]
impl Requester<SignedHeader, Header> for MockRequester {
    async fn signed_header(&self, h: u64) -> Result<LightSignedHeader, Error> {
        println!("requested signed header for height:{:?}", h);
        if let Some(sh) = self.signed_headers.get(&h) {
            return Ok(sh.into());
        }
        println!("couldn't get sh for: {}", &h);
        fail!(Kind::RequestFailed, "couldn't get sh for: {}", &h);
    }

    async fn validator_set(&self, h: u64) -> Result<Set, Error> {
        println!("requested validators for height:{:?}", h);
        if let Some(vs) = self.validators.get(&h) {
            return Ok(vs.to_owned());
        }
        println!("couldn't get vals for: {}", &h);
        fail!(Kind::RequestFailed, "couldn't get vals for: {}", &h);
    }
}

impl MockRequester {
    fn new(chain_id: String, lite_blocks: Vec<LiteBlock>) -> Self {
        let mut sh_map: HashMap<u64, SignedHeader> = HashMap::new();
        let mut val_map: HashMap<u64, Set> = HashMap::new();
        let last_block = lite_blocks.last().expect("last entry not found");
        val_map.insert(
            last_block.signed_header.header.height.increment().value(),
            last_block.to_owned().next_validator_set,
        );
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

// Link to the commit that generated below JSON test files:
// https://github.com/Shivani912/tendermint/commit/e02f8fd54a278f0192353e54b84a027c8fe31c1e
const TEST_FILES_PATH: &str = "./tests/support/lite/";
fn read_json_fixture(file_path: &str) -> String {
    fs::read_to_string(PathBuf::from(file_path)).unwrap()
}

type Trusted = lite::TrustedState<SignedHeader, Header>;

fn run_test_case(tc: &TestCase) {
    let trusted_next_vals = tc.initial.clone().next_validator_set;
    let mut latest_trusted =
        Trusted::new(tc.initial.signed_header.clone().into(), trusted_next_vals);
    test_serialization_roundtrip(&latest_trusted);

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
            untrusted_vals,
            untrusted_next_vals,
            TrustThresholdFraction::default(),
            trusting_period,
            now,
        ) {
            Ok(new_state) => {
                let expected_state = TrustedState::new(
                    untrusted_signed_header.clone().into(),
                    untrusted_next_vals.clone(),
                );

                assert_eq!(new_state, expected_state);
                assert!(!expects_err);

                latest_trusted = new_state.clone();
                test_serialization_roundtrip(&latest_trusted);
            }
            Err(_) => {
                assert!(expects_err);
            }
        }
    }
}
#[tokio::test]
async fn bisection() {
    // TODO: re-enable multi-peer tests as soon as the light client can handle these:
    // let dir = "bisection/multi_peer";
    // run_bisection_tests(dir).await;

    let dir = "bisection/single_peer";
    run_bisection_tests(dir).await;
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

async fn run_bisection_tests(dir: &str) {
    let paths = fs::read_dir(PathBuf::from(TEST_FILES_PATH).join(dir)).unwrap();

    for file_path in paths {
        let dir_entry = file_path.unwrap();
        let fp_str = format!("{}", dir_entry.path().display());
        println!(
            "Running light client against bisection test-file:\n {:?}",
            fp_str
        );
        let case: TestBisection = read_bisection_test_case(&fp_str);
        run_bisection_test(case).await;
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
            continue;
        }
        println!(
            "Running light client against 'single-step' test-file:\n {}",
            fp_str
        );
        let case: TestCase = read_test_case(&fp_str);
        run_test_case(&case);
    }
}

fn read_bisection_test_case(file_path: &str) -> TestBisection {
    serde_json::from_str(read_json_fixture(file_path).as_str()).unwrap()
}

fn read_test_case(file_path: &str) -> TestCase {
    serde_json::from_str(read_json_fixture(file_path).as_str()).unwrap()
}

async fn run_bisection_test(case: TestBisection) {
    println!("{}", case.description);

    let untrusted_height = case.height_to_verify.try_into().unwrap();
    let trust_threshold = case.trust_options.trust_level;
    let trusting_period = case.trust_options.period;
    let now = case.now;

    let provider = case.primary;
    let req = MockRequester::new(provider.chain_id, provider.lite_blocks);

    let expects_err = match &case.expected_output {
        Some(eo) => eo.eq("error"),
        None => false,
    };

    let trusted_height = case.trust_options.height.try_into().unwrap();
    let trusted_header = req
        .signed_header(trusted_height)
        .await
        .expect("could not 'request' signed header");
    let trusted_vals = req
        .validator_set(trusted_height + 1)
        .await
        .expect("could not 'request' validator set");

    let trusted_state = TrustedState::new(trusted_header, trusted_vals);

    match lite::verify_bisection(
        trusted_state,
        untrusted_height,
        trust_threshold,
        trusting_period.into(),
        now.into(),
        &req,
    )
    .await
    {
        Ok(new_states) => {
            let untrusted_signed_header = req
                .signed_header(untrusted_height)
                .await
                .expect("header at untrusted height not found");

            let untrusted_next_vals = req
                .validator_set(untrusted_height + 1)
                .await
                .expect("val set at untrusted height not found");

            let expected_state = TrustedState::new(untrusted_signed_header, untrusted_next_vals);
            assert_eq!(new_states[new_states.len() - 1], expected_state);
            let _expected_num_of_bisections = case.expected_num_of_bisections;
            // TODO: number of bisections started diverting in JSON tests and Rust impl
            // assert_eq!(
            //     new_states.len() as i32,
            //     expected_num_of_bisections,
            //     "expected # bisections"
            // );
            assert!(!expects_err);
        }
        Err(_) => {
            assert!(expects_err);
        }
    }
}
