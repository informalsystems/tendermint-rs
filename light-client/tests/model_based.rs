use serde::Deserialize;
use std::time::Duration;
use tendermint_light_client::components::verifier::Verdict;
use tendermint_light_client::{
    tests::{Trusted, *},
    types::{LightBlock, Time, TrustThreshold},
};
use tendermint_testgen::{apalache::*, jsonatr::*, Command, TestEnv, Tester};

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum LiteTestKind {
    SingleStep,
    Bisection,
}

/// An abstraction of the LightClient verification verdict
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum LiteVerdict {
    /// verified successfully
    #[serde(rename = "SUCCESS")]
    Success,
    /// outside of trusting period
    #[serde(rename = "FAILED_TRUSTING_PERIOD")]
    FailedTrustingPeriod,
    /// block verification based on the header and commit structure failed
    #[serde(rename = "INVALID")]
    Invalid,
    /// passed block verification, but the validator set is too different to verify it
    #[serde(rename = "NOT_ENOUGH_TRUST")]
    NotEnoughTrust,
}

/// A single-step test case is a test for `Verifier::verify()` function.
/// It contains an initial trusted block, plus a sequence of input blocks,
/// each with the expected verdict.
/// The trusted state is to be updated only if the verdict is "Success"
#[derive(Deserialize, Clone, Debug)]
pub struct SingleStepTestCase {
    description: String,
    initial: Initial,
    input: Vec<BlockVerdict>,
}

/// A LiteBlock together with the time when it's being checked, and the expected verdict
#[derive(Deserialize, Clone, Debug)]
pub struct BlockVerdict {
    block: AnonLightBlock,
    now: Time,
    verdict: LiteVerdict,
}

fn single_step_test(
    tc: SingleStepTestCase,
    _env: &TestEnv,
    _root_env: &TestEnv,
    output_env: &TestEnv,
) {
    let mut latest_trusted = Trusted::new(
        tc.initial.signed_header.clone(),
        tc.initial.next_validator_set.clone(),
    );
    let clock_drift = Duration::from_secs(1);
    let trusting_period: Duration = tc.initial.trusting_period.into();
    for (i, input) in tc.input.iter().enumerate() {
        output_env.logln(&format!("    > step {}, expecting {:?}", i, input.verdict));
        let now = input.now;
        match verify_single(
            latest_trusted.clone(),
            input.block.clone().into(),
            TrustThreshold::default(),
            trusting_period,
            clock_drift,
            now,
        ) {
            Ok(new_state) => {
                assert_eq!(input.verdict, LiteVerdict::Success);
                let expected_state: LightBlock = input.block.clone().into();
                assert_eq!(new_state, expected_state);
                latest_trusted = Trusted::new(new_state.signed_header, new_state.next_validators);
            }
            Err(e) => {
                output_env.logln(&format!("      > lite: {:?}", e));
                match e {
                    Verdict::Invalid(_) => assert_eq!(input.verdict, LiteVerdict::Invalid),
                    Verdict::NotEnoughTrust(_) => {
                        assert_eq!(input.verdict, LiteVerdict::NotEnoughTrust)
                    }
                    Verdict::Success => {
                        panic!("verify_single() returned error with Verdict::Success")
                    }
                }
            }
        }
    }
}

fn model_based_test(
    test: ApalacheTestCase,
    env: &TestEnv,
    root_env: &TestEnv,
    output_env: &TestEnv,
) {
    println!("  Running model-based single-step test case: {}", test.test);
    let check_program = |program| {
        if !Command::exists_program(program) {
            output_env.logln(&format!("    > {} not found", program));
            return false;
        }
        true
    };
    if !check_program("tendermint-testgen")
        || !check_program("apalache-mc")
        || !check_program("jsonatr")
    {
        output_env.logln("    failed to find necessary programs; consider adding them to your PATH. skipping the test");
        return;
    }
    env.copy_file_from_env(root_env, "Lightclient_A_1.tla");
    env.copy_file_from_env(root_env, "Blockchain_A_1.tla");
    env.copy_file_from_env(root_env, "LightTests.tla");
    env.copy_file_from_env(root_env, &test.model);

    println!("  > running Apalache...");
    match run_apalache_test(env.current_dir(), test) {
        Ok(run) => match run {
            ApalacheRun::Counterexample(_) => (),
            run => panic!(run.message().to_string()),
        },
        Err(e) => panic!("failed to run Apalache; reason: {}", e),
    }
    output_env.copy_file_from_env(env, "counterexample.tla");
    output_env.copy_file_from_env(env, "counterexample.json");

    let transform_spec = root_env
        .full_canonical_path("_jsonatr-lib/apalache_to_lite_test.json")
        .unwrap();
    let transform = JsonatrTransform {
        input: "counterexample.json".to_string(),
        include: vec![transform_spec],
        output: "test.json".to_string(),
    };
    assert!(run_jsonatr_transform(env.current_dir(), transform).is_ok());
    output_env.copy_file_from_env(env, "test.json");

    let tc: SingleStepTestCase = env.parse_file("test.json").unwrap();
    println!("  > running auto-generated test...");
    single_step_test(tc, env, root_env, output_env);
}

fn model_based_test_batch(batch: ApalacheTestBatch) -> Vec<(String, String)> {
    let mut res = Vec::new();
    for test in batch.tests {
        let tc = ApalacheTestCase {
            model: batch.model.clone(),
            test: test.clone(),
            length: batch.length,
            timeout: batch.timeout,
        };
        res.push((test.clone(), serde_json::to_string(&tc).unwrap()));
    }
    res
}

const TEST_DIR: &str = "./tests/support/model_based";

#[test]
fn run_model_based_single_step_tests() {
    let mut tester = Tester::new("single_step", TEST_DIR);
    tester.add_test_with_env("static model-based single-step test", single_step_test);
    tester.add_test_with_env("full model-based single-step test", model_based_test);
    tester.add_test_batch(model_based_test_batch);
    tester.run_foreach_in_dir("");
    tester.finalize();
}
