//! Node keys

use crate::error::{self, Error};
use crate::{node, private_key::PrivateKey, public_key::PublicKey};
use serde::{Deserialize, Serialize};
use _std::{fs, path::Path};

/// P2P node private keys
#[derive(Serialize, Deserialize)]
pub struct NodeKey {
    /// Private key
    pub priv_key: PrivateKey,
}

impl NodeKey {
    /// Parse `node_key.json`
    pub fn parse_json<T: AsRef<str>>(json_string: T) -> Result<Self, Error> {
        serde_json::from_str(json_string.as_ref()).map_err(error::serde_json_error)
    }

    /// Load `node_key.json` from a file
    pub fn load_json_file<P>(path: &P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let json_string = fs::read_to_string(path).map_err(|e| {
            let context = format!("couldn't open {}: {}", path.as_ref().display(), e);
            error::parse_error(context)
        })?;

        Self::parse_json(json_string)
    }

    /// Get the public key for this keypair
    pub fn public_key(&self) -> PublicKey {
        match &self.priv_key {
            PrivateKey::Ed25519(keypair) => keypair.public.into(),
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
