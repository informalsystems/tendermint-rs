use abscissa::{Callable, GlobalConfig};
use std::process;

use client::Client;
use config::{KmsConfig, ValidatorConfig};
use keyring::KeyRing;

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

        KeyRing::load_from_config(&config.providers).unwrap_or_else(|e| {
            status_err!("couldn't load keyring: {}", e);
            process::exit(1);
        });

        // Spawn the validator client threads
        let validator_clients = spawn_validator_clients(&config.validator);

        // Wait for the validator client threads to exit
        // TODO: Find something more useful for this thread to do
        for client in validator_clients {
            client.join();
        }
    }
}

/// Spawn validator client threads (which provide KMS service to the
/// validators they connect to)
fn spawn_validator_clients(config: &[ValidatorConfig]) -> Vec<Client> {
    config
        .iter()
        .map(|validator| Client::spawn(validator.clone()))
        .collect()
}
