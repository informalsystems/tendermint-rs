//! Configuration for the ed25519-dalek backend

use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct DalekConfig {
    /// Ed25519 private key configurations
    pub keys: BTreeMap<String, DalekPrivateKey>,
}

#[derive(Deserialize, Debug)]
pub struct DalekPrivateKey {
    /// Path to a file containing a cryptographic key
    pub path: PathBuf,
}
