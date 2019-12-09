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
    signed_header: SignedHeader,
    validator_set: Set,
    next_validator_set: Set,
}

pub struct DefaultTrustLevel {}
impl lite::TrustLevel for DefaultTrustLevel {}

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
            if let Err(e) = lite::expired(&trusted_signed_header.header, trusting_period, now) {
                println!("Expired: {:?}", e);
                assert_eq!(expexts_err, true);
            }
            let new_signed_header = &input.signed_header;
            let new_vals = input.validator_set.clone();
            let res = &lite::verify(
                trusted_signed_header.clone(),
                trusted_next_vals.clone(),
                new_signed_header.clone(),
                new_vals,
                DefaultTrustLevel {},
            );
            assert_eq!(res.is_err(), expexts_err);
            if !res.is_err() {
                trusted_signed_header = new_signed_header;
                trusted_next_vals = input.next_validator_set.clone();
            } else {
                println!("Got error: {:?}", res.as_ref().err());
            }
        }
    }
}

#[test]
fn verify_trusting_with_one_validator_no_changes() {
    let suite: TestSuite = serde_json::from_str(&read_json_fixture("basic")).unwrap();
    lite::verify_trusting(
        suite.signed_header.header.clone(),
        suite.signed_header,
        validator::Set::new(suite.last_validators),
        validator::Set::new(suite.validators),
        DefaultTrustLevel {},
    )
    .expect("verify_trusting failed");
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
