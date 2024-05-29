use std::{fs, path::PathBuf};

use tendermint_rpc::{endpoint, Response};

use walkdir::WalkDir;

fn find_fixtures(in_out_folder_name: &str) -> Vec<PathBuf> {
    WalkDir::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("dydx_fixtures")
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
            // NOTE: for the purpose of the test I manually emptied `validator_updates` from the
            // block_results, which has another unrelated issue deserializing the Secp256K1 public
            // key
            "block_results_at_height_12791634" => {
                let r = endpoint::block_results::Response::from_string(content);
                assert!(r.is_ok(), "block_results_at_height_12791634: {r:?}");
            },
            _ => {
                panic!("unhandled incoming fixture: {file_name}");
            },
        }
    }
}
