//! Node keys

use crate::{node, public_key::PublicKey};
use crate::{private_key::PrivateKey, Error, ErrorKind};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

/// P2P node private keys
#[derive(Serialize, Deserialize)]
pub struct NodeKey {
    /// Private key
    pub priv_key: PrivateKey,
}

impl NodeKey {
    /// Parse `node_key.json`
    pub fn parse_json<T: AsRef<str>>(json_string: T) -> Result<Self, Error> {
        Ok(serde_json::from_str(json_string.as_ref())?)
    }

    /// Load `node_key.json` from a file
    pub fn load_json_file<P>(path: &P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let json_string = fs::read_to_string(path).map_err(|e| {
            err!(
                ErrorKind::Parse,
                "couldn't open {}: {}",
                path.as_ref().display(),
                e
            )
        })?;

        Self::parse_json(json_string)
    }

    /// Get the public key for this keypair
    pub fn public_key(&self) -> PublicKey {
        match &self.priv_key {
            PrivateKey::Ed25519(key) => key.public_key(),
        }
    }

    // TODO(xla): Should return a Result or be implemented for all variants.
    /// Get node ID for this keypair
    pub fn node_id(&self) -> node::Id {
        match &self.public_key() {
            PublicKey::Ed25519(key) => node::Id::from(*key),
            PublicKey::Secp256k1(_pk) => unimplemented!(),
        }
    }
}
