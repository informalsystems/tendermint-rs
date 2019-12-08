use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs, path::PathBuf};
use tendermint::{lite, rpc::endpoint::commit::SignedHeader, validator};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TestSuite {
    signed_header: SignedHeader,
    last_validators: Vec<validator::Info>,
    validators: Vec<validator::Info>,
}

fn read_json_fixture(name: &str) -> String {
    fs::read_to_string(PathBuf::from("./tests/support/lite/").join(name.to_owned() + ".json"))
        .unwrap()
}

#[test]
fn verify_trusting_with_one_validator_no_changes() {
    pub struct DefaultTrustLevel {}
    impl lite::TrustLevel for DefaultTrustLevel {}

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
