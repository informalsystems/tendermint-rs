//! Validator private keys

use crate::public_key::TendermintKey;
use crate::{
    account,
    error::{Error, Kind},
    private_key::PrivateKey,
    public_key::PublicKey,
};
use anomaly::format_err;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

/// Validator private key
#[derive(Serialize, Deserialize)] // JSON custom serialization for priv_validator_key.json
pub struct PrivValidatorKey {
    /// Address
    pub address: account::Id,

    /// Public key
    pub pub_key: PublicKey,

    /// Private key
    pub priv_key: PrivateKey,
}

impl PrivValidatorKey {
    /// Parse `priv_validator_key.json`
    pub fn parse_json<T: AsRef<str>>(json_string: T) -> Result<Self, Error> {
        let result = serde_json::from_str::<Self>(json_string.as_ref())?;

        // Validate that the parsed key type is usable as a consensus key
        TendermintKey::new_consensus_key(result.priv_key.public_key())?;

        Ok(result)
    }

    /// Load `node_key.json` from a file
    pub fn load_json_file<P>(path: &P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let json_string = fs::read_to_string(path).map_err(|e| {
            format_err!(
                Kind::Parse,
                "couldn't open {}: {}",
                path.as_ref().display(),
                e
            )
        })?;

        Self::parse_json(json_string)
    }

    /// Get the consensus public key for this validator private key
    pub fn consensus_pubkey(&self) -> TendermintKey {
        TendermintKey::new_consensus_key(self.priv_key.public_key()).unwrap()
    }
}
