//! Key Management System for Cosmos Validators

extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
extern crate signatory;
#[macro_use]
extern crate structopt;
extern crate toml;

mod config;
mod error;
mod ed25519;

use std::fs::File;
use std::io::Read;
use structopt::StructOpt;

use config::Config;

/// Command line arguments (using structopt as the parser)
#[derive(StructOpt, Debug)]
#[structopt(name = "kms", about = "Key Management System for Cosmos Validators")]
enum Opts {
    #[structopt(name = "run", about = "")]
    Run {
        /// Path to configuration file
        #[structopt(short = "c", long = "config", default_value = "kms.toml")]
        config: String,

        /// Print debugging information
        #[structopt(short = "v", long = "verbose")]
        verbose: bool,
    },
}

/// Main entry point
fn main() {
    match Opts::from_args() {
        Opts::Run { config, verbose } => run(&config, verbose),
    }
}

/// Run the KMS
fn run(config_file: &str, _verbose: bool) {
    let _config = read_config(config_file);
}

/// Parse the configuration TOML, returning a Config struct
fn read_config(filename: &str) -> Config {
    let mut file =
        File::open(filename).unwrap_or_else(|e| panic!("couldn't open {}: {}", filename, e));

    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    toml::from_str(&data).unwrap_or_else(|e| panic!("couldn't parse {}: {:?}", filename, e))
}
