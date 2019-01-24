use abscissa::{Callable, GlobalConfig};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

use crate::{
    client::Client,
    config::{KmsConfig, ValidatorConfig},
    keyring::KeyRing,
};

/// The `start` command
#[derive(Debug, Options)]
pub struct StartCommand {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    pub config: Option<String>,

    /// Print debugging information
    #[options(short = "v", long = "verbose")]
    pub verbose: bool,
}

impl Default for StartCommand {
    fn default() -> Self {
        Self {
            config: None,
            verbose: false,
        }
    }
}

impl Callable for StartCommand {
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

        // Should we terminate yet?
        let should_term = Arc::new(AtomicBool::new(false));
        // Spawn the validator client threads
        let validator_clients = spawn_validator_clients(&config.validator, &should_term);
        let catch_signals = [signal_hook::SIGTERM, signal_hook::SIGINT];

        // Listen for the relevant signals so we can gracefully shut down
        for sig in catch_signals.iter() {
            signal_hook::flag::register(*sig, Arc::clone(&should_term)).unwrap_or_else(|e| {
                status_err!("couldn't register signal hook: {}", e);
                process::exit(1);
            });
        }

        // Keep checking in on whether or not we need to terminate
        while !should_term.load(Ordering::Relaxed) {
            thread::sleep(time::Duration::from_millis(100));
        }

        // Wait for all of the validator client threads to exit
        info!("Waiting for client threads to stop...");
        for client in validator_clients {
            client.join();
        }
    }
}

/// Spawn validator client threads (which provide KMS service to the
/// validators they connect to)
fn spawn_validator_clients(
    config: &[ValidatorConfig],
    should_term: &Arc<AtomicBool>,
) -> Vec<Client> {
    config
        .iter()
        .map(|validator| Client::spawn(validator.clone(), Arc::clone(should_term)))
        .collect()
}
