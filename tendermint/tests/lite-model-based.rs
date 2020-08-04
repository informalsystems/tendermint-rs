use std::{process, io};
use std::io::Read;
use std::{fs, path::PathBuf};
use serde::Deserialize;
use tempfile::tempdir;
use tendermint::{block::signed_header::SignedHeader, evidence::Duration, lite, Hash, Time};
use tendermint::block::{Header, Height};
use tendermint::lite::{Requester, TrustThresholdFraction, TrustedState};
mod lite_tests;
use lite_tests::*;

type Trusted = lite::TrustedState<SignedHeader, Header>;

/// A test case is the initial trusted block,
/// plus a sequence of input blocks, each with the expected result
#[derive(Deserialize, Clone, Debug)]
pub struct ApalacheTests {
    description: String,
    model: String,
    length_bound: Option<u64>,
    timeout: Option<u64>,
    tests: Vec<String>,
}

/// A test case is the initial trusted block,
/// plus a sequence of input blocks, each with the expected verdict
/// The trusted state is to be updated only if the verdict is "OK"
#[derive(Deserialize, Clone, Debug)]
pub struct SingleTestCase {
    description: String,
    initial: Initial,
    input: Vec<BlockVerdict>,
}

/// A LiteBlock together with the expected verdict
#[derive(Deserialize, Clone, Debug)]
pub struct BlockVerdict {
    block: LiteBlock,
    verdict: String,
}

const TEST_DIR: &str = "./tests/support/lite-model-based/";

fn read_file(dir: &str, file: &str) -> String {
    fs::read_to_string(PathBuf::from(dir.to_owned() + file)).unwrap()
}

fn read_single_test_case(dir: &str, file: &str) -> SingleTestCase {
    serde_json::from_str(read_file(dir, file).as_str()).unwrap()
}

fn run_single_test_case(tc: &SingleTestCase) {
    let trusted_next_vals = tc.initial.clone().next_validator_set;
    let mut latest_trusted =
        Trusted::new(tc.initial.signed_header.clone().into(), trusted_next_vals);
    test_serialization_roundtrip(&latest_trusted);

    let trusting_period: std::time::Duration = tc.initial.clone().trusting_period.into();
    let tm_now = tc.initial.now;
    let now = tm_now.to_system_time().unwrap();

    for (i, input) in tc.input.iter().enumerate() {
        println!("i: {}, {}", i, tc.description);

        let untrusted_signed_header = &input.block.signed_header;
        let untrusted_vals = &input.block.validator_set;
        let untrusted_next_vals = &input.block.next_validator_set;

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
                assert_eq!(input.verdict, "OK");
                let expected_state = TrustedState::new(
                    untrusted_signed_header.clone().into(),
                    untrusted_next_vals.clone(),
                );
                assert_eq!(new_state, expected_state);
                latest_trusted = new_state.clone();
                test_serialization_roundtrip(&latest_trusted);
            }
            Err(e) => {
                assert_ne!(input.verdict, "OK");
            }
        }
    }
}

#[test]
fn apalache() {
    let tc = read_single_test_case(TEST_DIR, "first-model-based-test.json");
    run_test_case(&tc);
}
