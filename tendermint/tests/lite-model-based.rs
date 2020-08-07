use serde::Deserialize;
//use tempfile::tempdir;
use tendermint::{block::signed_header::SignedHeader, lite};
use tendermint::block::{Header};
use tendermint::lite::{TrustThresholdFraction, TrustedState};

mod utils;
use utils::{*, apalache::*, jsonatr::*, command::*, lite::*};

type Trusted = lite::TrustedState<SignedHeader, Header>;

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum LiteTestKind {
    SingleStep,
    Bisection
}

/// An abstraction of the LightClient verification verdict
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum LiteVerdict {
    /// verified successfully
    OK,
    /// outside of trusting period
    FAILED_TRUSTING_PERIOD,
    /// block verification based on the header and commit structure failed
    FAILED_VERIFICATION,
    /// passed block verification, but the validator set is too different to verify it
    CANNOT_VERIFY
}

/// A single-step test case is a test for `lite::verify_single()` function.
/// It contains an initial trusted block, plus a sequence of input blocks,
/// each with the expected verdict.
/// The trusted state is to be updated only if the verdict is "OK"
#[derive(Deserialize, Clone, Debug)]
pub struct SingleStepTestCase {
    description: String,
    initial: Initial,
    input: Vec<BlockVerdict>,
}

/// A LiteBlock together with the expected verdict
#[derive(Deserialize, Clone, Debug)]
pub struct BlockVerdict {
    block: LiteBlock,
    verdict: LiteVerdict,
}

const TEST_DIR: &str = "./tests/support/lite-model-based/";

fn read_single_step_test(dir: &str, file: &str) -> SingleStepTestCase {
    serde_json::from_str(read_file(dir, file).as_str()).unwrap()
}

fn run_single_step_test(tc: &SingleStepTestCase) {
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
                assert_eq!(input.verdict, LiteVerdict::OK);
                let expected_state = TrustedState::new(
                    untrusted_signed_header.clone().into(),
                    untrusted_next_vals.clone(),
                );
                assert_eq!(new_state, expected_state);
                latest_trusted = new_state.clone();
                test_serialization_roundtrip(&latest_trusted);
            }
            Err(e) => {
                assert_ne!(input.verdict, LiteVerdict::OK);
            }
        }
    }
}

#[test]
fn single_step_test() {
    let tc = read_single_step_test(TEST_DIR, "first-model-based-test.json");
    run_single_step_test(&tc);
}

#[test]
fn apalache_test() {
    assert!(Command::exists_program("tendermint-testgen"));
    assert!(Command::exists_program("apalache-mc"));
    assert!(Command::exists_program("jsonatr"));

    let test = ApalacheTestCase {
        model: "MC4_4_faulty.tla".to_string(),
        test: "TestFailureInv".to_string(),
        length: None,
        timeout: None
    };
    assert!(run_apalache_test(TEST_DIR, test).is_ok());

    let transform = JsonatrTransform {
        input: "counterexample.json".to_string(),
        include: vec!["../../utils/jsonatr-lib/apalache_to_lite_test.json".to_string()],
        output: "lite_test.json".to_string()
    };
    assert!(run_jsonatr_transform(TEST_DIR, transform).is_ok());

    let tc = read_single_step_test(TEST_DIR, "lite_test.json");
    run_single_step_test(&tc);
}
