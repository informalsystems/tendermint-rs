use super::*;
use crate::yubihsm::get_hsm_client;
use abscissa::Callable;
use serde_json::Value;
use signatory::ed25519;
use std::{fs::File, io::prelude::*, process, str};
use subtle_encoding::base64;
use tendermint::public_keys::ConsensusKey;

/// The `yubihsm keys import` subcommand
#[derive(Debug, Default, Options)]
pub struct ImportCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    pub config: Option<String>,

    /// Path to the validator configuration file
    #[options(short = "p", long = "path")]
    pub path: Option<String>,

    /// Type of key to import (default 'ed25519')
    #[options(short = "t")]
    pub key_type: Option<String>,

    /// Label for imported key
    #[options(short = "l", long = "label")]
    pub label: Option<String>,

    /// Key ID for imported key
    #[options(free)]
    key_id: Option<u16>,
}

impl Callable for ImportCommand {
    fn call(&self) {
        if self.path.is_none() {
            status_err!("must provide a valid path to priv_validator.json");
            process::exit(1);
        }

        if self.key_id.is_none() {
            status_err!("must provide a unique key_id");
            process::exit(1);
        }

        match &self.key_type {
            Some(ref key_type) => {
                if key_type != DEFAULT_KEY_TYPE {
                    status_err!(
                        "only supported key type is: ed25519 (given: \"{}\")",
                        key_type
                    );
                    process::exit(1);
                }
            }
            None => (),
        }

        if let Some(path) = &self.path {
            let mut f = File::open(path).unwrap_or_else(|e| {
                status_err!("couldn't open validator config file {}: {}", path, e);
                process::exit(1);
            });

            let mut contents = Vec::new();
            f.read_to_end(&mut contents).unwrap_or_else(|e| {
                status_err!("couldn't read validator config file {}: {}", path, e);
                process::exit(1);
            });
            let v: Value = serde_json::from_slice(&contents).unwrap();
            let s = v["priv_key"]["value"].as_str().unwrap_or_else(|| {
                status_err!("couldn't read validator private key from config: {}", path);
                process::exit(1);
            });

            let key_pair = base64::decode(s).unwrap_or_else(|e| {
                status_err!("couldn't decode validator private key from config: {}", e);
                process::exit(1);
            });
            let seed = ed25519::Seed::from_keypair(&key_pair).unwrap_or_else(|e| {
                status_err!("invalid key in validator config: {}", e);
                process::exit(1);
            });
            let key = seed.as_secret_slice();
            let label =
                yubihsm::object::Label::from(self.label.as_ref().map(|l| l.as_ref()).unwrap_or(""));

            let mut hsm = get_hsm_client();

            if let Err(e) = hsm.put_asymmetric_key(
                self.key_id.unwrap(),
                label,
                DEFAULT_DOMAINS,
                DEFAULT_CAPABILITIES,
                yubihsm::AsymmetricAlg::Ed25519,
                key,
            ) {
                status_err!("couldn't import key #{}: {}", self.key_id.unwrap(), e);
                process::exit(1);
            }

            let public_key = ed25519::PublicKey::from_bytes(
                hsm.get_public_key(self.key_id.unwrap())
                    .unwrap_or_else(|e| {
                        status_err!(
                            "couldn't get public key for key #{}: {}",
                            self.key_id.unwrap(),
                            e
                        );
                        process::exit(1);
                    }),
            )
            .unwrap();

            status_ok!(
                "Imported",
                "key #{}: {}",
                self.key_id.unwrap(),
                ConsensusKey::from(public_key)
            );
        }
    }
}

impl_command!(ImportCommand);
