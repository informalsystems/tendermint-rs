use abscissa::{Callable, GlobalConfig};
use signatory::{self, Decode, Ed25519Seed, Encode};
use signatory_dalek::Ed25519Signer;
use std::process;

use client::Client;
use config::{KmsConfig, SecretConnectionConfig, ValidatorConfig};
use ed25519::{KeyRing, PublicKey, SECRET_KEY_ENCODING};
use error::{KmsError, KmsErrorKind::*};

/// The `run` command
#[derive(Debug, Options)]
pub struct RunCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    pub config: Option<String>,

    /// Print debugging information
    #[options(short = "v", long = "verbose")]
    pub verbose: bool,
}

impl Default for RunCommand {
    fn default() -> Self {
        Self {
            config: None,
            verbose: false,
        }
    }
}

impl Callable for RunCommand {
    /// Run the KMS
    fn call(&self) {
        info!(
            "{} {} starting up...",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        let config = KmsConfig::get_global();

        let secret_connection_key = load_secret_connection_key(&config.secret_connection)
            .unwrap_or_else(|e| {
                status_err!("couldn't load secret connection key: {}", e);
                process::exit(1);
            });

        log_kms_node_id(&secret_connection_key);

        KeyRing::load_from_config(&config.providers).unwrap_or_else(|e| {
            status_err!("couldn't load keyring: {}", e);
            process::exit(1);
        });

        // Spawn the validator client threads
        let validator_clients = spawn_validator_clients(&config.validator, &secret_connection_key);

        // Wait for the validator client threads to exit
        // TODO: Find something more useful for this thread to do
        for client in validator_clients {
            client.join();
        }
    }
}

/// Initialize KMS secret connection private key
fn load_secret_connection_key(config: &SecretConnectionConfig) -> Result<Ed25519Seed, KmsError> {
    let key_path = &config.secret_key_path;

    if key_path.exists() {
        Ok(Ed25519Seed::decode_from_file(key_path, SECRET_KEY_ENCODING)
            .map_err(|e| err!(ConfigError, "error loading {}: {}", key_path.display(), e))?)
    } else {
        let seed = Ed25519Seed::generate();
        seed.encode_to_file(key_path, SECRET_KEY_ENCODING)?;
        Ok(seed)
    }
}

/// Log the KMS node ID
fn log_kms_node_id(seed: &Ed25519Seed) {
    let public_key = PublicKey::from(signatory::public_key(&Ed25519Signer::from(seed)).unwrap());
    info!("{} node ID: {}", env!("CARGO_PKG_NAME"), &public_key);
}

/// Spawn validator client threads (which provide KMS service to the
/// validators they connect to)
fn spawn_validator_clients(
    config: &[ValidatorConfig],
    secret_connection_key: &Ed25519Seed,
) -> Vec<Client> {
    config
        .iter()
        .map(|validator| Client::spawn(validator.clone(), secret_connection_key.clone()))
        .collect()
}
