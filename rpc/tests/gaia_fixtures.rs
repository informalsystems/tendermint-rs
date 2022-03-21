use std::fs;
use std::path::PathBuf;
use tendermint_rpc::event::Event;
use tendermint_rpc::{endpoint, Request, Response};
use walkdir::WalkDir;

fn find_fixtures(in_out_folder_name: &str) -> Vec<PathBuf> {
    WalkDir::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("gaia_fixtures")
            .join(in_out_folder_name),
    )
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| {
        e.file_type().is_file()
            && e.path().extension().is_some()
            && e.path().extension().unwrap() == "json"
    })
    .map(|e| e.into_path())
    .collect::<Vec<PathBuf>>()
}

#[test]
fn incoming_fixtures() {
    for json_file in find_fixtures("incoming") {
        let file_name = json_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(".json")
            .unwrap();
        let content = fs::read_to_string(&json_file).unwrap();
        match file_name {
            "abci_info" => {
                let r = endpoint::abci_info::Response::from_string(content);
                assert!(r.is_ok(), "{:?}", r)
            }
            "block_at_height_0" => {
                assert!(endpoint::block::Response::from_string(content).is_err())
            }
            "block_at_height_1" => {
                assert!(endpoint::block::Response::from_string(content).is_ok())
            }
            "block_at_height_10" => {
                assert!(endpoint::block::Response::from_string(content).is_ok())
            }
            "block_at_height_4555980" => {
                let r = endpoint::block::Response::from_string(content);
                assert!(r.is_ok(), "{:?}", r);
            }
            "block_results_at_height_10" => {
                let r = endpoint::block_results::Response::from_string(content);
                assert!(r.is_ok(), "block_results_at_height_10: {:?}", r);
            }
            "block_results_at_height_4555980" => {
                let r = endpoint::block_results::Response::from_string(content);
                assert!(r.is_ok(), "block_results_at_height_4555980: {:?}", r);
            }
            "blockchain_from_1_to_10" => {
                assert!(endpoint::blockchain::Response::from_string(content).is_ok())
            }
            "commit_at_height_10" => {
                assert!(endpoint::commit::Response::from_string(content).is_ok())
            }
            "consensus_params" => {
                assert!(endpoint::consensus_params::Response::from_string(content).is_ok())
            }
            "consensus_state" => {
                assert!(endpoint::consensus_state::Response::from_string(content).is_ok())
            }
            "genesis" => {
                assert!(
                    endpoint::genesis::Response::<serde_json::Value>::from_string(content).is_ok()
                )
            }
            "net_info" => {
                assert!(endpoint::net_info::Response::from_string(content).is_ok())
            }
            "status" => {
                assert!(endpoint::status::Response::from_string(content).is_ok())
            }
            "subscribe_newblock" => {
                let r = endpoint::subscribe::Response::from_string(content);
                assert!(r.is_err(), "{:?}", r);
            }
            _ => {
                if file_name.starts_with("subscribe_newblock_") {
                    let r = Event::from_string(content);
                    assert!(r.is_ok(), "failed to parse event {}: {:?}", file_name, r);
                } else {
                    panic!("unhandled incoming fixture: {}", file_name);
                }
            }
        }
    }
}

#[test]
fn outgoing_fixtures() {
    for json_file in find_fixtures("outgoing") {
        let file_name = json_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(".json")
            .unwrap();
        let content = fs::read_to_string(&json_file).unwrap();
        match file_name {
            "abci_info" => {
                let r = endpoint::abci_info::Request::from_string(content);
                assert!(r.is_ok(), "{:?}", r)
            }
            "block_at_height_0" => {
                assert!(endpoint::block::Request::from_string(content).is_ok())
            }
            "block_at_height_1" => {
                assert!(endpoint::block::Request::from_string(content).is_ok())
            }
            "block_at_height_10" => {
                assert!(endpoint::block::Request::from_string(content).is_ok())
            }
            "block_at_height_4555980" => {
                assert!(endpoint::block::Request::from_string(content).is_ok())
            }
            "block_results_at_height_10" => {
                let r = endpoint::block_results::Request::from_string(content);
                assert!(r.is_ok(), "block_results_at_height_10: {:?}", r);
            }
            "block_results_at_height_4555980" => {
                let r = endpoint::block_results::Request::from_string(content);
                assert!(r.is_ok(), "block_results_at_height_4555980: {:?}", r);
            }
            "blockchain_from_1_to_10" => {
                assert!(endpoint::blockchain::Request::from_string(content).is_ok())
            }
            "commit_at_height_10" => {
                assert!(endpoint::commit::Request::from_string(content).is_ok())
            }
            "consensus_params" => {
                assert!(endpoint::consensus_params::Request::from_string(content).is_ok())
            }
            "consensus_state" => {
                assert!(endpoint::consensus_state::Request::from_string(content).is_ok())
            }
            "genesis" => {
                assert!(
                    endpoint::genesis::Request::<Option<serde_json::Value>>::from_string(content)
                        .is_ok()
                )
            }
            "net_info" => {
                assert!(endpoint::net_info::Request::from_string(content).is_ok())
            }
            "status" => {
                assert!(endpoint::status::Request::from_string(content).is_ok())
            }
            "subscribe_newblock" => {
                let r = endpoint::subscribe::Request::from_string(content);
                assert!(r.is_ok(), "{:?}", r);
            }
            _ => panic!("unhandled outgoing fixture: {}", file_name),
        }
    }
}
