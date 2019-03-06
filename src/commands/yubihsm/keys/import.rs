use super::*;
use abscissa::Callable;
use signatory::ed25519;
use std::{fs, path::PathBuf, process};
use subtle_encoding::base64;
use tendermint::public_keys::ConsensusKey;
use yubihsm::object;

/// The `yubihsm keys import` subcommand
#[derive(Debug, Default, Options)]
pub struct ImportCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    pub config: Option<String>,

    /// ID of the key to import (if applicable)
    #[options(short = "i", long = "id")]
    pub key_id: Option<u16>,

    /// ID of the wrap key the original key was encrypted with
    #[options(short = "w", long = "wrapkey")]
    pub wrap_key_id: Option<u16>,

    /// Type of key to import (either `wrap` or `priv_validator`, default `wrap`)
    #[options(short = "t")]
    pub key_type: Option<String>,

    /// Label for imported key (only applicable to `priv_validator` keys)
    #[options(short = "l", long = "label")]
    pub label: Option<String>,

    /// Path to the key to import
    #[options(free)]
    pub path: PathBuf,
}

impl Callable for ImportCommand {
    fn call(&self) {
        let contents = fs::read_to_string(&self.path).unwrap_or_else(|e| {
            status_err!("couldn't import file {}: {}", self.path.display(), e);
            process::exit(1);
        });

        match self.key_type.as_ref().map(|ty| ty.as_str()) {
            Some("wrap") => self.import_wrapped(&contents),
            Some("priv_validator") => self.import_priv_validator_json(&contents),
            Some(other) => {
                status_err!("invalid key type: {}", other);
                process::exit(1);
            }
            None => {
                if self.path.ends_with("priv_validator.json") {
                    self.import_priv_validator_json(&contents)
                } else {
                    self.import_wrapped(&contents)
                }
            }
        }
    }
}

impl ImportCommand {
    /// Import a wrapped object into the HSM
    fn import_wrapped(&self, wrapped_key_base64: &str) {
        if let Some(id) = self.key_id {
            status_warn!("ignoring key ID: {} (wrapped keys use original key ID)", id);
        }

        if let Some(ref label) = self.label {
            status_warn!(
                "ignoring label: {:?} (wrapped keys use original label)",
                label
            );
        }

        let wrap_key_id = self.wrap_key_id.unwrap_or(DEFAULT_WRAP_KEY);

        let wrapped_key_ciphertext =
            base64::decode(wrapped_key_base64.as_bytes()).unwrap_or_else(|e| {
                status_err!(
                    "couldn't decode Base64-encoded wrapped key from {}: {}",
                    self.path.display(),
                    e
                );
                process::exit(1);
            });

        let wrapped_message = yubihsm::wrap::Message::from_vec(wrapped_key_ciphertext)
            .unwrap_or_else(|e| {
                status_err!(
                    "couldn't parse wrapped key from {}: {}",
                    self.path.display(),
                    e
                );
                process::exit(1);
            });

        let hsm = crate::yubihsm::client();

        let obj = hsm
            .import_wrapped(wrap_key_id, wrapped_message)
            .unwrap_or_else(|e| {
                status_err!(
                    "error importing encrypted key from {} (using wrapkey 0x{:04x}): {}",
                    self.path.display(),
                    wrap_key_id,
                    e
                );
                process::exit(1);
            });

        if obj.object_type != object::Type::AsymmetricKey {
            // We mainly care about importing asymmetric keys, but can handle other types.
            // If we encounter a non-asymmetric key type, display basic info.
            status_ok!(
                "Imported",
                "object 0x{:04x} ({:?})",
                obj.object_id,
                obj.object_type
            );
            process::exit(0);
        }

        let public_key = hsm.get_public_key(obj.object_id).unwrap_or_else(|e| {
            status_err!(
                "couldn't get public key for asymmetric key #{}: {}",
                obj.object_id,
                e
            );
            process::exit(1);
        });

        // TODO: support for non-Cosmos keys, non-Ed25519 keys, non-validator (i.e. account) keys
        let key_info = match public_key.algorithm {
            yubihsm::asymmetric::Algorithm::Ed25519 => {
                ConsensusKey::from(ed25519::PublicKey::from_bytes(&public_key.as_ref()).unwrap())
                    .to_string()
            }
            alg => format!("{:?}: {:?}", alg, public_key.as_ref()),
        };

        status_ok!("Imported", "key 0x{:04x}: {}", obj.object_id, key_info);
    }

    /// Import an existing priv_validator file into the HSM
    // TODO(tarcieri): ideally this can eventually be removed. Its value seems time-limited
    // and it makes this module much more complex than functionality for importing wrapped backups
    fn import_priv_validator_json(&self, json_data: &str) {
        if let Some(id) = self.wrap_key_id {
            status_warn!(
                "ignoring wrapkey ID: {} (not applicable to priv_validator.json files)",
                id
            );
        }

        let key_id = self.key_id.unwrap_or_else(|| {
            status_err!(
                "no key ID specified (use e.g. tmkms yubihsm keys import -i 1 priv_validator.json)"
            );
            process::exit(1);
        });

        let v: serde_json::Value = serde_json::from_str(json_data).unwrap();

        let s = v["priv_key"]["value"].as_str().unwrap_or_else(|| {
            status_err!(
                "couldn't read validator private key from config: {}",
                self.path.display()
            );
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

        if let Err(e) = crate::yubihsm::client().put_asymmetric_key(
            key_id,
            label,
            DEFAULT_DOMAINS,
            DEFAULT_CAPABILITIES,
            yubihsm::asymmetric::Algorithm::Ed25519,
            key,
        ) {
            status_err!("couldn't import key #{}: {}", self.key_id.unwrap(), e);
            process::exit(1);
        }

        status_ok!("Imported", "key 0x{:04x}", key_id);
    }
}

impl_command!(ImportCommand);
