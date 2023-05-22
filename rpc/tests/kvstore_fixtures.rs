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
    mod v0_38;
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
