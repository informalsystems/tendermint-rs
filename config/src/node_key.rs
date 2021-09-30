//! Node keys

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tendermint::{node, private_key::PrivateKey, public_key::PublicKey};

use crate::error::Error;

/// P2P node private keys
#[derive(Serialize, Deserialize)]
pub struct NodeKey {
    /// Private key
    pub priv_key: PrivateKey,
}

impl NodeKey {
    /// Parse `node_key.json`
    pub fn parse_json<T: AsRef<str>>(json_string: T) -> Result<Self, Error> {
        let res = serde_json::from_str(json_string.as_ref()).map_err(Error::serde_json)?;
        Ok(res)
    }

    /// Load `node_key.json` from a file
    pub fn load_json_file<P>(path: &P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let json_string = fs::read_to_string(path)
            .map_err(|e| Error::file_io(format!("{}", path.as_ref().display()), e))?;

        Self::parse_json(json_string)
    }

    /// Get the public key for this keypair
    pub fn public_key(&self) -> PublicKey {
        #[allow(unreachable_patterns)]
        match &self.priv_key {
            PrivateKey::Ed25519(keypair) => keypair.public.into(),
            _ => unreachable!(),
        }
    }

    /// Get node ID for this keypair
    pub fn node_id(&self) -> node::Id {
        #[allow(unreachable_patterns)]
        match &self.public_key() {
            PublicKey::Ed25519(pubkey) => node::Id::from(*pubkey),
            _ => unreachable!(),
        }
    }
}
