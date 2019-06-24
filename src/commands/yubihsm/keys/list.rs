//! List keys inside the YubiHSM2

use crate::{application::app_config, chain, keyring};
use abscissa::{Command, Runnable};
use std::{collections::BTreeMap as Map, process};
use tendermint::{PublicKey, TendermintKey};

/// The `yubihsm keys list` subcommand
#[derive(Command, Debug, Default, Options)]
pub struct ListCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config", help = "path to tmkms.toml")]
    pub config: Option<String>,
}

impl Runnable for ListCommand {
    /// List all suitable Ed25519 keys in the HSM
    fn run(&self) {
        let key_formatters = load_key_formatters();
        let hsm = crate::yubihsm::client();

        let serial_number = hsm
            .device_info()
            .unwrap_or_else(|e| {
                status_err!("couldn't get YubiHSM serial number: {}", e);
                process::exit(1);
            })
            .serial_number;

        let objects = hsm.list_objects(&[]).unwrap_or_else(|e| {
            status_err!("couldn't list YubiHSM objects: {}", e);
            process::exit(1);
        });

        let mut keys = objects
            .iter()
            .filter(|o| o.object_type == yubihsm::object::Type::AsymmetricKey)
            .collect::<Vec<_>>();

        keys.sort_by(|k1, k2| k1.object_id.cmp(&k2.object_id));

        if keys.is_empty() {
            status_err!("no keys in this YubiHSM (#{})", serial_number);
            process::exit(0);
        }

        println!("Listing keys in YubiHSM #{}:", serial_number);

        for key in &keys {
            let public_key = hsm.get_public_key(key.object_id).unwrap_or_else(|e| {
                status_err!(
                    "couldn't get public key for asymmetric key #{}: {}",
                    key.object_id,
                    e
                );
                process::exit(1);
            });

            let key_id = format!("- 0x#{:04x}", key.object_id);

            // TODO: support for non-Ed25519 keys
            if public_key.algorithm != yubihsm::asymmetric::Algorithm::Ed25519 {
                status_attr_err!(key_id, "unsupported algorithm: {:?}", public_key.algorithm);
                continue;
            }

            // TODO: support for account keys
            let tendermint_key = TendermintKey::ConsensusKey(
                PublicKey::from_raw_ed25519(&public_key.as_ref()).unwrap(),
            );

            let public_key_serialized = match key_formatters.get(&key.object_id) {
                Some(key_formatter) => key_formatter.serialize(tendermint_key),
                None => tendermint_key.to_hex(),
            };

            status_attr_ok!(key_id, public_key_serialized);
        }
    }
}

/// Load information about configured YubiHSM keys
fn load_key_formatters() -> Map<u16, keyring::Format> {
    let chain_formatters = load_chain_formatters();
    let cfg = crate::yubihsm::config();
    let mut map = Map::new();

    for key_config in &cfg.keys {
        // Only use a preferred formatting if there is one chain per key
        if key_config.chain_ids.len() == 1 {
            if let Some(formatter) = chain_formatters.get(&key_config.chain_ids[0]) {
                if map.insert(key_config.key, formatter.clone()).is_some() {
                    status_err!("duplicate YubiHSM config for key: 0x{:04x}", key_config.key);
                    process::exit(1);
                }
            }
        }
    }

    map
}

/// Load chain-specific key formatters from the configuration
fn load_chain_formatters() -> Map<chain::Id, keyring::Format> {
    let cfg = app_config();
    let mut map = Map::new();

    for chain in &cfg.chain {
        if map.insert(chain.id, chain.key_format.clone()).is_some() {
            status_err!("duplicate chain config for '{}'", chain.id);
            process::exit(1);
        }
    }

    map
}
