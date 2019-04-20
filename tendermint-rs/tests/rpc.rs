//! Tendermint RPC tests

#[cfg(feature = "rpc")]
mod endpoints {
    use std::{fs, path::PathBuf};
    use tendermint::rpc::{endpoint::StatusResponse, Response};

    fn read_json_fixture(name: &str) -> String {
        fs::read_to_string(PathBuf::from("./tests/support/").join(name.to_owned() + ".json"))
            .unwrap()
    }

    #[test]
    fn status() {
        let status_json = read_json_fixture("status");
        let status_response = StatusResponse::from_json(&status_json).unwrap();

        assert_eq!(status_response.node_info.network.as_str(), "cosmoshub-1");
        assert_eq!(
            status_response.sync_info.latest_block_height.value(),
            410744
        );
        assert_eq!(status_response.validator_info.voting_power.value(), 0);
    }
}
