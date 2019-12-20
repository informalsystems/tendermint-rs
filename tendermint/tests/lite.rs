use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json;
use std::{fs, path::PathBuf};
use tendermint::lite::TrustedState;
use tendermint::{block::signed_header::SignedHeader, lite, validator, validator::Set, Time};

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
impl lite::TrustThreshold for DefaultTrustLevel {}

const TEST_FILES_PATH: &str = "./tests/support/lite/";
fn read_json_fixture(name: &str) -> String {
    fs::read_to_string(PathBuf::from(TEST_FILES_PATH).join(name.to_owned() + ".json")).unwrap()
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

struct Trusted {
    last_signed_header: SignedHeader,
    validators: Set,
}

impl lite::TrustedState for Trusted {
    type SignedHeader = SignedHeader;
    type ValidatorSet = Set;

    fn new(last_header: Self::SignedHeader, vals: Self::ValidatorSet) -> Self {
        Self {
            last_signed_header: last_header,
            validators: vals,
        }
    }

    fn last_signed_header(&self) -> &Self::SignedHeader {
        &self.last_signed_header
    }

    fn validators(&self) -> &Self::ValidatorSet {
        &self.validators
    }
}

fn run_test_cases(cases: TestCases) {
    for (_, tc) in cases.test_cases.iter().enumerate() {
        let trusted_next_vals = tc.initial.clone().next_validator_set;
        let mut trusted_state = Trusted::new(tc.initial.signed_header.clone(), trusted_next_vals);
        let trusting_period: std::time::Duration = tc.initial.clone().trusting_period.into();
        let now = tc.initial.now;
        let expects_err = match &tc.expected_output {
            Some(eo) => eo.eq("error"),
            None => false,
        };

        for (_, input) in tc.input.iter().enumerate() {
            println!("{}", tc.description);
            let new_signed_header = &input.signed_header;
            let new_vals = &input.validator_set;
            // Note that in the provided test files the other header is either assumed to
            // be "trusted" (verification already happened), or, it's the signed header verified in
            // the previous iteration of this loop. In both cases it is assumed that h1 was already
            // verified.
            let h2_verif_res = lite::verify(new_signed_header, new_vals);
            let mut check_support_res: Result<(), lite::Error> = Ok(());
            if h2_verif_res.is_ok() {
                check_support_res = lite::check_support(
                    &trusted_state,
                    &new_signed_header,
                    &DefaultTrustLevel {},
                    &trusting_period,
                    &now.into(),
                );
                assert_eq!(check_support_res.is_err(), expects_err);
                if check_support_res.is_ok() {
                    trusted_state =
                        Trusted::new(new_signed_header.clone(), input.next_validator_set.clone());
                }
            }
            let got_err = check_support_res.is_err() || h2_verif_res.is_err();
            assert_eq!(expects_err, got_err);
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
