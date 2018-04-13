//! Key Management System for Cosmos Validators

extern crate clear_on_drop;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate gumdrop;
#[macro_use]
extern crate gumdrop_derive;
#[macro_use]
extern crate log;
extern crate rand;
extern crate simplelog;
#[macro_use]
extern crate serde_derive;
extern crate signatory;
extern crate toml;

use gumdrop::Options;
use simplelog::{CombinedLogger, LevelFilter, TermLogger};
use simplelog::Config as LoggingConfig;
use std::collections::BTreeMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;

#[macro_use]
mod macros;

mod client;
mod config;
mod error;
mod ed25519;
mod session;

use clear_on_drop::ClearOnDrop;
use client::Client;
use config::{Config, ProviderConfig, ValidatorConfig};
use rand::{OsRng, Rng};
use ed25519::Keyring;

/// Unix file permissions required for private keys (i.e. owner-readable only)
pub const PRIVATE_KEY_PERMISSIONS: u32 = 0o600;

/// Command line arguments (using gumdrop as the parser)
#[derive(Debug, Options)]
enum Opts {
    #[options(help = "show help for a command")]
    Help(HelpOpts),

    #[cfg(feature = "dalek-provider")]
    #[options(help = "generate a new signing key")]
    Keygen(KeygenOpts),

    #[options(help = "run the KMS application")]
    Run(RunOpts),
}

/// Options for the `help` command
#[derive(Debug, Default, Options)]
struct HelpOpts {
    #[options(free)]
    free: Vec<String>,
}

/// Options for the `keygen` command
#[cfg(feature = "dalek-provider")]
#[derive(Debug, Default, Options)]
struct KeygenOpts {
    #[options(free)]
    path: Vec<PathBuf>,
}

/// Options for the `run` command
#[derive(Debug, Options)]
struct RunOpts {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    config: PathBuf,

    /// Print debugging information
    #[options(short = "v", long = "verbose")]
    verbose: bool,
}

impl Default for RunOpts {
    fn default() -> Self {
        Self {
            config: "kms.toml".into(),
            verbose: false,
        }
    }
}

/// Main entry point
fn main() {
    let args: Vec<_> = env::args().collect();

    let opts = Opts::parse_args_default(&args[1..]).unwrap_or_else(|e| {
        match e.to_string().as_ref() {
            // Show usage if no command name is given or if "help" is given
            "missing command name" => help(),
            string => eprintln!("{}: {}", args[0], string),
        }

        exit(2);
    });

    match opts {
        Opts::Help(_commands) => help(),
        #[cfg(feature = "dalek-provider")]
        Opts::Keygen(opts) => keygen(opts.path.as_ref()),
        Opts::Run(opts) => run(&opts.config, opts.verbose),
    }

    exit(0);
}

/// Print help message
fn help() {
    println!("Usage: {} [COMMAND] [OPTIONS]", env::args().next().unwrap());
    println!();
    println!("Available commands:");
    println!();
    println!("{}", Opts::command_list().unwrap());
    println!();
}

/// Generate an Ed25519 secret key for use with a software provider (i.e. ed25519-dalek)
#[cfg(feature = "dalek-provider")]
fn keygen(output_paths: &[PathBuf]) {
    init_logging(true);

    if output_paths.len() != 1 {
        eprintln!("Usage: {} keygen [PATH]", env::args().next().unwrap());
        exit(2);
    }

    let output_path = &output_paths[0];

    // Buffer which will receive the random seed value
    let mut seed = ClearOnDrop::new(vec![0u8; 32]);
    OsRng::new().unwrap().fill_bytes(seed.as_mut());

    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(PRIVATE_KEY_PERMISSIONS)
        .open(output_path)
        .unwrap_or_else(|e| {
            error!("couldn't open {} for writing: {}", output_path.display(), e);
            exit(1);
        });

    // TODO: some sort of serialization format for the private key? Raw is easy for now
    output_file.write_all(&*seed).unwrap_or_else(|e| {
        error!("couldn't write to {}: {}", output_path.display(), e);
        exit(1);
    });

    info!(
        "Wrote random Ed25519 private key to {}",
        output_path.display()
    );
}

/// Run the KMS
fn run(config_file: &Path, verbose: bool) {
    init_logging(verbose);

    info!(
        "{} {} starting up...",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let Config {
        validators,
        providers,
    } = load_config(config_file);

    let keyring = Arc::new(init_keyring(providers));

    // Spawn the validator client threads
    let validator_clients = spawn_validator_clients(validators, &keyring);

    // Wait for the validator client threads to exit
    // TODO: Find something more useful for this thread to do
    for client in validator_clients {
        client.join();
    }
}

/// Initialize the logger
fn init_logging(verbose: bool) {
    let level_filter = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    CombinedLogger::init(vec![
        TermLogger::new(level_filter, LoggingConfig::default()).unwrap(),
    ]).unwrap();
}

/// Load the configuration file
fn load_config(config_file: &Path) -> Config {
    Config::load(config_file).unwrap_or_else(|e| {
        error!("error reading {}: {}", config_file.display(), e);
        exit(1);
    })
}

/// Initialize the keyring
fn init_keyring(config: ProviderConfig) -> Keyring {
    Keyring::from_config(config).unwrap_or_else(|e| {
        error!("signer error: {}", e);
        exit(1);
    })
}

/// Spawn the validator clients (which expose the KMS "service")
fn spawn_validator_clients(
    config: BTreeMap<String, ValidatorConfig>,
    keyring: &Arc<Keyring>,
) -> Vec<Client> {
    config
        .iter()
        .map(|(label, validator_config)| {
            Client::spawn(
                label.clone(),
                validator_config.addr.clone(),
                validator_config.port,
                Arc::clone(&keyring),
            )
        })
        .collect()
}
