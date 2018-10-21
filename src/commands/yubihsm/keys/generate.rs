use abscissa::Callable;
use std::process;

use ed25519::{ConsensusKey, PublicKey};
use yubihsm;

/// Default key type to generate
pub const DEFAULT_KEY_TYPE: &str = "ed25519";

/// Default YubiHSM2 domain (internal partitioning)
pub const DEFAULT_DOMAINS: yubihsm::Domain = yubihsm::Domain::DOM1;

/// Default YubiHSM2 permissions for generated keys
pub const DEFAULT_CAPABILITIES: yubihsm::Capability = yubihsm::Capability::ASYMMETRIC_SIGN_EDDSA;

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

        match &self.key_type {
            Some(ref key_type) => if key_type != DEFAULT_KEY_TYPE {
                status_err!(
                    "only supported key type is: ed25519 (given: \"{}\")",
                    key_type
                );
                process::exit(1);
            },
            None => (),
        }

        let mut hsm = yubihsm::get_hsm_client();

        for key_id in &self.key_ids {
            let label =
                yubihsm::ObjectLabel::from(self.label.as_ref().map(|l| l.as_ref()).unwrap_or(""));

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

            let public_key = PublicKey::from(hsm.get_pubkey(*key_id).unwrap_or_else(|e| {
                status_err!("couldn't get public key for key #{}: {}", key_id, e);
                process::exit(1);
            }));

            status_ok!("Generated", "key #{}: {}", key_id, ConsensusKey(public_key));
        }
    }
}

// TODO: custom derive in abscissa
impl_command!(GenerateCommand);
