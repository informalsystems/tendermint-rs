use rand::Rng;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;
use std::time::Duration;
use tendermint_light_client::components::verifier::Verdict;
use tendermint_light_client::{
    tests::{Trusted, *},
    types::{LightBlock, Time, TrustThreshold},
};
use tendermint_testgen::{apalache::*, jsonatr::*, Command, TestEnv, Tester, Validator, Generator};

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum LiteTestKind {
    SingleStep,
    Bisection,
}

/// An abstraction of the LightClient verification verdict
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SingleStepTestCase {
    description: String,
    initial: Initial,
    input: Vec<BlockVerdict>,
}

/// A LiteBlock together with the time when it's being checked, and the expected verdict
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockVerdict {
    block: AnonLightBlock,
    now: Time,
    verdict: LiteVerdict,
}

/// A primitive fuzzer that mutates one of the test case input blocks
trait SingleStepTestFuzzer {
    fn fuzz(tc: &SingleStepTestCase) -> Option<SingleStepTestCase> {
        let mut fuzz = tc.clone();
        if let Some((i, input)) = Self::fuzzable_input(&mut fuzz) {
            let what = Self::fuzz_input(input);
            input.verdict = LiteVerdict::Invalid;
            fuzz.input.truncate(i + 1);
            fuzz.description = format!("Fuzzed {} for {}", what, &fuzz.description);
            return Some(fuzz);
        }
        None
    }

    /// Fuzzes the input block and returns the description of what was changed
    fn fuzz_input(input: &mut BlockVerdict) -> String;

    /// Get a random hash value
    fn random_hash() -> tendermint::hash::Hash {
        // not really random, but should be different from whatever hash is there
        tendermint::hash::Hash::from_str(
            "AAAAAAAAAA1BA22917BBE036BA9D58A40918E93983B57BD0DC465301E10B5419",
        )
        .unwrap()
    }

    /// get the block to mutate, if possible
    fn fuzzable_input(tc: &mut SingleStepTestCase) -> Option<(usize, &mut BlockVerdict)> {
        let mut indices = Vec::new();
        for (i, input) in tc.input.iter_mut().enumerate() {
            if input.verdict != LiteVerdict::Invalid {
                indices.push(i)
            }
        }
        if indices.is_empty() {
            None
        } else {
            let mut rng = rand::thread_rng();
            let i = rng.gen_range(0, indices.len());
            Some((i, tc.input.get_mut(i).unwrap()))
        }
    }
}

struct HeaderVersionFuzzer {}
impl SingleStepTestFuzzer for HeaderVersionFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        let mut rng = rand::thread_rng();
        let mut block = input.block.signed_header.header.version.block;
        while block == input.block.signed_header.header.version.block {
            block = rng.gen();
        }
        let mut app = input.block.signed_header.header.version.app;
        while app == input.block.signed_header.header.version.app {
            app = rng.gen();
        }
        input.block.signed_header.header.version = tendermint::block::header::Version{ block, app };
        String::from("header version")
    }
}

struct HeaderChainIdFuzzer {}
impl SingleStepTestFuzzer for HeaderChainIdFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        input.block.signed_header.header.chain_id =
            tendermint::chain::Id::from_str("AAAAAAAAAAAAAAAAAA").unwrap();
        String::from("header chain_id")
    }
}

struct HeaderHeightFuzzer {}
impl SingleStepTestFuzzer for HeaderHeightFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        let mut rng = rand::thread_rng();
        let h: u64 = input.block.signed_header.header.height.into();
        let mut height: u64 = rng.gen_range(0u64, i64::MAX as u64);
        while height == h {
            height = rng.gen();
        }
        input.block.signed_header.header.height =
            tendermint::block::Height::try_from(height).unwrap();
        String::from("header height")
    }
}

struct HeaderTimeFuzzer {}
impl SingleStepTestFuzzer for HeaderTimeFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        let mut rng = rand::thread_rng();
        let secs = tendermint::Time::now()
            .duration_since(tendermint::Time::unix_epoch())
            .unwrap()
            .as_secs();
        let rand_secs = rng.gen_range(0, secs);
        input.block.signed_header.header.time =
            tendermint::Time::unix_epoch() + std::time::Duration::from_secs(rand_secs);
        // TODO: the fuzzing below fails with one of:
        //   - 'overflow when adding duration to instant', src/libstd/time.rs:549:31
        //   - 'No such local time', /home/andrey/.cargo/registry/src/github.com-1ecc6299db9ec823/chrono-0.4.11/src/offset/mod.rs:173:34
        // let secs: u64 = rng.gen();
        // input.block.signed_header.header.time = tendermint::Time::unix_epoch() + std::time::Duration::from_secs(secs);
        String::from("header time")
    }
}

struct HeaderLastBlockIdFuzzer {}
impl SingleStepTestFuzzer for HeaderLastBlockIdFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        input.block.signed_header.header.last_block_id =
            Some(tendermint::block::Id {
                hash: Self::random_hash(),
                parts: Default::default()
            });
        String::from("header last_block_id")
    }
}

struct HeaderLastCommitHashFuzzer {}
impl SingleStepTestFuzzer for HeaderLastCommitHashFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        input.block.signed_header.header.last_commit_hash = Some(Self::random_hash());
        String::from("header last_commit_hash")
    }
}

struct HeaderDataHashFuzzer {}
impl SingleStepTestFuzzer for HeaderDataHashFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        input.block.signed_header.header.data_hash = Some(Self::random_hash());
        String::from("header data_hash")
    }
}

struct HeaderValHashFuzzer {}
impl SingleStepTestFuzzer for HeaderValHashFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        input.block.signed_header.header.validators_hash = Self::random_hash();
        String::from("header validators_hash")
    }
}

struct HeaderNextValHashFuzzer {}
impl SingleStepTestFuzzer for HeaderNextValHashFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        input.block.signed_header.header.next_validators_hash = Self::random_hash();
        String::from("header next_validators_hash")
    }
}

struct HeaderConsensusHashFuzzer {}
impl SingleStepTestFuzzer for HeaderConsensusHashFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        input.block.signed_header.header.consensus_hash = Self::random_hash();
        String::from("header consensus_hash")
    }
}

struct HeaderAppHashFuzzer {}
impl SingleStepTestFuzzer for HeaderAppHashFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        input.block.signed_header.header.app_hash =
            tendermint::hash::AppHash::try_from(vec![0,1,2,3,4,5]).unwrap();
        String::from("header app_hash")
    }
}

struct HeaderLastResultsHashFuzzer {}
impl SingleStepTestFuzzer for HeaderLastResultsHashFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        input.block.signed_header.header.last_results_hash = Some(Self::random_hash());
        String::from("header last_results_hash")
    }
}

struct HeaderEvidenceHashFuzzer {}
impl SingleStepTestFuzzer for HeaderEvidenceHashFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        input.block.signed_header.header.evidence_hash = Some(Self::random_hash());
        String::from("header evidence_hash")
    }
}

struct HeaderProposerAddressFuzzer {}
impl SingleStepTestFuzzer for HeaderProposerAddressFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        let val = Validator::new("AAAAAAAAAAAAAAAA");
        input.block.signed_header.header.proposer_address = val.generate().unwrap().address;
        String::from("header proposer_address")
    }
}

struct CommitHeightFuzzer {}
impl SingleStepTestFuzzer for CommitHeightFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        let mut rng = rand::thread_rng();
        let h: u64 = input.block.signed_header.commit.height.into();
        let mut height: u64 = h  - 2; // rng.gen_range(0u64, i64::MAX as u64);
        while height == h {
            height = rng.gen();
        }
        input.block.signed_header.commit.height =
            tendermint::block::Height::try_from(h).unwrap();
        String::from("commit height")
    }
}

struct CommitRoundFuzzer {}
impl SingleStepTestFuzzer for CommitRoundFuzzer {
    fn fuzz_input(input: &mut BlockVerdict) -> String {
        let mut rng = rand::thread_rng();
        let r: u32 = input.block.signed_header.commit.round.into();
        let mut round: u32 = rng.gen();
        while round == r {
            round = rng.gen();
        }
        input.block.signed_header.commit.round = round;
        format!("commit round from {} into {}", r, round)
    }
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

fn fuzz_single_step_test(
    tc: SingleStepTestCase,
    _env: &TestEnv,
    _root_env: &TestEnv,
    output_env: &TestEnv,
) {
    output_env.clear_log();
    let run_test = |tc: SingleStepTestCase| {
        output_env.logln(&format!(
            "  > running static model-based single-step test: {}",
            &tc.description
        ));
        single_step_test(tc, _env, _root_env, output_env);
        Some(())
    };
    run_test(tc.clone());
    HeaderVersionFuzzer::fuzz(&tc).and_then(run_test);
    HeaderChainIdFuzzer::fuzz(&tc).and_then(run_test);
    HeaderHeightFuzzer::fuzz(&tc).and_then(run_test);
    HeaderTimeFuzzer::fuzz(&tc).and_then(run_test);
    HeaderLastBlockIdFuzzer::fuzz(&tc).and_then(run_test);
    HeaderLastCommitHashFuzzer::fuzz(&tc).and_then(run_test);
    HeaderDataHashFuzzer::fuzz(&tc).and_then(run_test);
    HeaderValHashFuzzer::fuzz(&tc).and_then(run_test);
    HeaderNextValHashFuzzer::fuzz(&tc).and_then(run_test);
    HeaderConsensusHashFuzzer::fuzz(&tc).and_then(run_test);
    HeaderAppHashFuzzer::fuzz(&tc).and_then(run_test);
    HeaderLastResultsHashFuzzer::fuzz(&tc).and_then(run_test);
    HeaderEvidenceHashFuzzer::fuzz(&tc).and_then(run_test);
    HeaderProposerAddressFuzzer::fuzz(&tc).and_then(run_test);
    // The tests below fail -- seems that there is not enough validation between the header and the commit
    //CommitHeightFuzzer::fuzz(&tc).and_then(run_test);
    //CommitRoundFuzzer::fuzz(&tc).and_then(run_test);
}

fn model_based_test(
    test: ApalacheTestCase,
    env: &TestEnv,
    root_env: &TestEnv,
    output_env: &TestEnv,
) {
    println!("  Running model-based single-step test: {}", test.test);
    let tla_test = format!(
        "{}_{}.tla",
        test.model.strip_suffix(".tla").unwrap(),
        &test.test
    );
    let json_test = format!(
        "{}_{}.json",
        test.model.strip_suffix(".tla").unwrap(),
        &test.test
    );

    // Cleanup possible previous runs
    output_env.clear_log();
    output_env.remove_file(&tla_test);
    output_env.remove_file(&json_test);

    // Check for the necessary programs
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
    env.copy_file_from_env(root_env, "Lightclient_002_draft.tla");
    env.copy_file_from_env(root_env, "Blockchain_002_draft.tla");
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
    output_env.copy_file_from_env_as(env, "counterexample.tla", &tla_test);

    let transform_spec = root_env
        .full_canonical_path("_jsonatr-lib/apalache_to_lite_test.json")
        .unwrap();
    let transform = JsonatrTransform {
        input: "counterexample.json".to_string(),
        include: vec![transform_spec],
        output: "test.json".to_string(),
    };
    assert!(run_jsonatr_transform(env.current_dir(), transform).is_ok());
    output_env.copy_file_from_env_as(env, "test.json", &json_test);

    let mut tc: SingleStepTestCase = env.parse_file("test.json").unwrap();
    tc.description = json_test.clone();
    output_env.write_file(json_test, &serde_json::to_string_pretty(&tc).unwrap());
    fuzz_single_step_test(tc, env, root_env, output_env);
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
    let mut tester = Tester::new("test_run", TEST_DIR);
    tester.add_test_with_env("static model-based single-step test", fuzz_single_step_test);
    tester.add_test_with_env("full model-based single-step test", model_based_test);
    tester.add_test_batch(model_based_test_batch);
    tester.run_foreach_in_dir("");
    tester.finalize();
}
