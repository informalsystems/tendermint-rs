use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json;
use std::{fs, path::PathBuf};
use tendermint::validator::Set;
use tendermint::{lite, rpc::endpoint::commit::SignedHeader, validator, Time};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TestSuite {
    signed_header: SignedHeader,
    last_validators: Vec<validator::Info>,
    validators: Vec<validator::Info>,
}

#[derive(Clone, Debug)]
struct Duration(u64);

#[derive(Deserialize, Clone, Debug)]
struct TestCases {
    test_cases: Vec<TestCase>,
}

#[derive(Deserialize, Clone, Debug)]
struct TestCase {
    test_name: String,
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
    // TODO: feedback for shivani:
    // wouldn't a lite block structure be better of the form:
    // - a height h
    // - one reply of a /commit?height=h
    // - one reply of a /validators?height=h
    // ?
    // Then for tests this structure could be used for the skipping case too:
    // The light client get's two such LiteBlocks (one with height h and one with height h-x)
    // This gives more flexibility on how to compose the tests and can be used
    // to mock both rpc endpoints too?
    signed_header: SignedHeader,
    validator_set: Set,
    next_validator_set: Set,
}

pub struct DefaultTrustLevel {}
impl lite::TrustThreshold for DefaultTrustLevel {}

fn read_json_fixture(name: &str) -> String {
    fs::read_to_string(PathBuf::from("./tests/support/lite/").join(name.to_owned() + ".json"))
        .unwrap()
}

#[test]
fn val_set_tests_verify() {
    let cases: TestCases = serde_json::from_str(&read_json_fixture("val_set_tests")).unwrap();
    run_test_cases(cases);
}

#[test]
fn commit_tests_verify() {
    let cases: TestCases = serde_json::from_str(&read_json_fixture("commit_tests")).unwrap();
    run_test_cases(cases);
}

#[test]
fn header_tests_verify() {
    let cases: TestCases = serde_json::from_str(&read_json_fixture("header_tests")).unwrap();
    run_test_cases(cases);
}

fn run_test_cases(cases: TestCases) {
    for (_, tc) in cases.test_cases.iter().enumerate() {
        let mut trusted_signed_header = &tc.initial.signed_header;
        let mut trusted_next_vals = tc.initial.clone().next_validator_set;
        let trusting_period: std::time::Duration = tc.initial.clone().trusting_period.into();
        let now = tc.initial.now;
        let expexts_err = match &tc.expected_output {
            Some(eo) => eo.eq("error"),
            None => false,
        };

        for (_, input) in tc.input.iter().enumerate() {
            println!("{}", tc.description);
            let new_signed_header = &input.signed_header;
            let new_vals = &input.validator_set;
            // note that in the provided test files the other header is either assumed to
            // be "trusted" (verification already happened), or, it's the signed header verifier in
            // the previous iteration of this loop...
            let h2_verif_res = lite::verify(new_signed_header, new_vals);
            let mut check_support_res: Result<(), lite::Error> = Ok(());
            if h2_verif_res.is_ok() {
                check_support_res = lite::check_support(
                    trusted_signed_header,
                    &trusted_next_vals,
                    &new_signed_header,
                    DefaultTrustLevel {},
                    trusting_period,
                    now.into(),
                );
                assert_eq!(check_support_res.is_err(), expexts_err);
                if check_support_res.is_ok() {
                    trusted_signed_header = new_signed_header;
                    trusted_next_vals = input.next_validator_set.clone();
                }
            }
            let got_err = check_support_res.is_err() || h2_verif_res.is_err();
            assert_eq!(expexts_err, got_err);
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
