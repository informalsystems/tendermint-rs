use serde::Deserialize;
//use tempfile::tempdir;
use tendermint::{block::signed_header::SignedHeader, lite, Time};
use tendermint::block::{Header};
use tendermint::lite::{TrustThresholdFraction, TrustedState};

mod utils;
use utils::{*, apalache::*, jsonatr::*, command::*, lite::*};
use std::{fs, path::PathBuf};

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
    #[serde(rename = "FAILED_TRUSTING_PERIOD")]
    FailedTrustingPeriod,
    /// block verification based on the header and commit structure failed
    #[serde(rename = "FAILED_VERIFICATION")]
    FailedVerification,
    /// passed block verification, but the validator set is too different to verify it
    #[serde(rename = "CANNOT_VERIFY")]
    CannotVerify
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

/// A LiteBlock together with the time when it's being checked, and the expected verdict
#[derive(Deserialize, Clone, Debug)]
pub struct BlockVerdict {
    block: LiteBlock,
    now: Time,
    verdict: LiteVerdict,
}

const TEST_DIR: &str = "./tests/support/lite-model-based/";
const RUN_DIR: &str = "./tests/support/lite-model-based/last-run";

fn read_single_step_test(dir: &str, file: &str) -> Option<SingleStepTestCase> {
    let file = read_file(dir, file);
    parse_as::<SingleStepTestCase>(&file)
}

fn run_single_step_test(tc: &SingleStepTestCase) {
    let trusted_next_vals = tc.initial.clone().next_validator_set;
    let mut latest_trusted =
        Trusted::new(tc.initial.signed_header.clone().into(), trusted_next_vals);
    test_serialization_roundtrip(&latest_trusted);

    let trusting_period: std::time::Duration = tc.initial.clone().trusting_period.into();

    for (i, input) in tc.input.iter().enumerate() {
        println!("    > step {}, expecting {:?}", i, input.verdict);
        let tm_now = input.now;
        let now = tm_now.to_system_time().unwrap();
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
                eprintln!("      > lite: {}", e.to_string());
                assert_ne!(input.verdict, LiteVerdict::OK);
            }
        }
    }
}

fn check_program(program: &str) -> bool {
    if !Command::exists_program(program) {
        println!("  > {} not found", program);
        return false
    }
    true
}

fn copy_into(file: &str) -> bool {
    if !fs::copy(PathBuf::from(TEST_DIR).join(file),
             PathBuf::from(RUN_DIR).join(file)).is_ok() {
        println!("  > failed to copy file {} into the run directory", file);
        return false
    }
    true
}

fn run_model_based_test(test: &ApalacheTestCase) {
    if !check_program("tendermint-testgen") ||
       !check_program("apalache-mc") ||
       !check_program("jsonatr") {
        return
    }
    if !copy_into(&test.model) ||
       !copy_into("LiteTests.tla") ||
       !copy_into("Lightclient_A_1.tla") ||
       !copy_into("Blockchain_A_1.tla") {
        return
    }
    println!("  > running Apalache...");
    let apalache_run = run_apalache_test(RUN_DIR, test);
    assert!(apalache_run.is_ok());
    if !apalache_run.unwrap().stdout.contains("The outcome is: Error") {
        println!("  > Apalache failed to generate a counterexample; please check the model, the test, and the length bound");
    }
    else {
        let transform = JsonatrTransform {
            input: "counterexample.json".to_string(),
            include: vec!["../../../utils/jsonatr-lib/apalache_to_lite_test.json".to_string()],
            output: "lite_test.json".to_string()
        };
        assert!(run_jsonatr_transform(RUN_DIR, transform).is_ok());

        let tc= read_single_step_test(RUN_DIR, "lite_test.json");
        assert!(tc.is_some());
        println!("  > running auto-generated test...");
        run_single_step_test(&tc.unwrap());
    }
}

#[test]
fn run_single_step_tests() {
    let paths = fs::read_dir(PathBuf::from(TEST_DIR)).unwrap();
    for path in paths {
        if let Ok(entry) = path {
            if let Ok(kind) = entry.file_type() {
                if kind.is_file() || kind.is_symlink() {
                    let path = format!("{}", entry.path().display());
                    if !path.ends_with(".json") {
                        continue
                    }
                    let file = read_file("", &path);
                    if let Some(tc) = parse_as::<SingleStepTestCase>(&file) {
                        println!("Running static single-step test case: {}", path);
                        run_single_step_test(&tc);
                    }
                    else if let Some(tc) = parse_as::<ApalacheTestCase>(&file) {
                        println!("Running model-based single-step test case: {}", path);
                        run_model_based_test(&tc);
                    }
                    else if let Some(batch) = parse_as::<ApalacheTestBatch>(&file) {
                        println!("Running model-based single-step test batch: {}", path);
                        println!("{}", batch.description);
                        for test in batch.tests {
                            let tc = ApalacheTestCase {
                                model: batch.model.clone(),
                                test: test.clone(),
                                length: batch.length,
                                timeout: batch.timeout
                            };
                            println!("  Running model-based single-step test case: {}", test);
                            run_model_based_test(&tc);
                        }
                    }
                }
            }

        }
    }
}
