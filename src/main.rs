//! Key Management System for Cosmos Validators

extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate gumdrop;
#[macro_use]
extern crate gumdrop_derive;
#[macro_use]
extern crate log;
extern crate simplelog;
#[macro_use]
extern crate serde_derive;
extern crate signatory;
extern crate toml;

use simplelog::{CombinedLogger, LevelFilter, TermLogger};
use simplelog::Config as LoggingConfig;
use std::{env, process};

#[macro_use]
mod macros;

mod client;
mod config;
mod error;
mod ed25519;
mod session;

use client::Client;
use config::Config;
use gumdrop::Options;

/// Command line arguments (using gumdrop as the parser)
#[derive(Debug, Options)]
enum Opts {
    #[options(help = "show help for a command")]
    Help(HelpOpts),

    #[options(help = "run the KMS application")]
    Run(RunOpts),
}

/// Options for the `help` command
#[derive(Debug, Default, Options)]
struct HelpOpts {
    #[options(free)]
    free: Vec<String>,
}

/// Options for the `run` command
#[derive(Debug, Options)]
struct RunOpts {
    /// Path to configuration file
    #[options(short = "c", long = "config")]
    config: String,

    /// Print debugging information
    #[options(short = "v", long = "verbose")]
    verbose: bool,
}

impl Default for RunOpts {
    fn default() -> Self {
        Self {
            config: "kms.toml".to_owned(),
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

        process::exit(2);
    });

    match opts {
        Opts::Run(opts) => run(&opts.config, opts.verbose),
        Opts::Help(_commands) => help(),
    }
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

/// Run the KMS
fn run(config_file: &str, verbose: bool) {
    let config = Config::load(config_file);
    init_logging(verbose);

    info!(
        "{} {} starting up...",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    // Spawn the validator client threads
    let client_handles: Vec<Client> = config
        .validators
        .iter()
        .map(|(_, validator_config)| {
            Client::spawn(validator_config.addr.clone(), validator_config.port)
        })
        .collect();

    // Wait for the validator client threads to exit
    for handle in client_handles {
        handle.join();
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
