use super::*;
use crate::yubihsm::get_hsm_client;
use abscissa::Callable;
use signatory::ed25519;
use std::process;
use tendermint::public_keys::ConsensusKey;

/// The `yubihsm keys generate` subcommand
#[derive(Debug, Default, Options)]
pub struct GenerateCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    pub config: Option<String>,

    /// Label for generated key(s)
    #[options(short = "l", long = "label")]
    pub label: Option<String>,

    /// Type of key to generate (default 'ed25519')
    #[options(short = "t")]
    pub key_type: Option<String>,

    /// Key IDs to generate
    #[options(free)]
    key_ids: Vec<u16>,
}

impl Callable for GenerateCommand {
    /// Generate an Ed25519 signing key inside a YubiHSM2 device
    fn call(&self) {
        if self.key_ids.is_empty() {
            status_err!("must provide at least one key ID to generate");
            process::exit(1);
        }

        if let Some(key_type) = self.key_type.as_ref() {
            if key_type != DEFAULT_KEY_TYPE {
                status_err!(
                    "only supported key type is: ed25519 (given: \"{}\")",
                    key_type
                );
                process::exit(1);
            }
        }

        let mut hsm = get_hsm_client();

        for key_id in &self.key_ids {
            let label =
                yubihsm::object::Label::from(self.label.as_ref().map(|l| l.as_ref()).unwrap_or(""));

            if let Err(e) = hsm.generate_asymmetric_key(
                *key_id,
                label,
                DEFAULT_DOMAINS,
                DEFAULT_CAPABILITIES,
                yubihsm::AsymmetricAlg::Ed25519,
            ) {
                status_err!("couldn't generate key #{}: {}", key_id, e);
                process::exit(1);
            }

            let public_key =
                ed25519::PublicKey::from_bytes(hsm.get_public_key(*key_id).unwrap_or_else(|e| {
                    status_err!("couldn't get public key for key #{}: {}", key_id, e);
                    process::exit(1);
                }))
                .unwrap();

            status_ok!(
                "Generated",
                "key #{}: {}",
                key_id,
                ConsensusKey::from(public_key)
            );
        }
    }
}

// TODO: custom derive in abscissa
impl_command!(GenerateCommand);
