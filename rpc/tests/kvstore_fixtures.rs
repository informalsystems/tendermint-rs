//! Tendermint kvstore RPC endpoint testing.

use core::str::FromStr;
use std::{collections::BTreeMap as HashMap, fs, path::PathBuf};

use subtle_encoding::{base64, hex};
use tendermint::{
    abci,
    evidence::{Duration, Evidence},
    hash::Algorithm,
    public_key,
    vote::Vote,
    Hash,
};
use tendermint_config::net::Address;
use tendermint_rpc::{
    endpoint,
    error::{Error, ErrorDetail},
    request::Wrapper as RequestWrapper,
    Code, Order, Response,
};
use walkdir::WalkDir;

const CHAIN_ID: &str = "dockerchain";

// Test modules in the kvstore_fixtures subdirectory
mod kvstore_fixtures {
    use super::*;
    mod v0_34;
    mod v0_37;
}

fn find_fixtures(ver_folder_name: &str, in_out_folder_name: &str) -> Vec<PathBuf> {
    WalkDir::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("kvstore_fixtures")
            .join(ver_folder_name)
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

fn check_event_attrs(events: &HashMap<String, Vec<String>>, app_key: &str, height: i64) {
    for (k, v) in events {
        assert_eq!(v.len(), 1);
        match k.as_str() {
            "app.creator" => assert_eq!(v[0], "Cosmoshi Netowoko"),
            "app.index_key" => assert_eq!(v[0], "index is working"),
            "app.key" => assert_eq!(v[0], app_key),
            "app.noindex_key" => assert_eq!(v[0], "index is working"),
            "tm.event" => assert_eq!(v[0], "Tx"),
            "tx.hash" => assert_eq!(v[0].len(), 64),
            "tx.height" => assert_eq!(v[0], height.to_string()),
            _ => panic!("unknown event found {k}"),
        }
    }
}
