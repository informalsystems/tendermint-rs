//! `start` subcommand - start the light node.

/// App-local prelude includes `app_reader()`/`app_writer()`/`app_config()`
/// accessors along with logging macros. Customize as you see fit.
use abscissa_core::{config, Command, FrameworkError, Options, Runnable};
use std::process;

use tendermint::lite::{Height, Requester, TrustThresholdFraction};

use crate::application::APPLICATION;
use crate::config::LightNodeConfig;
use crate::prelude::*;

use abscissa_core::path::PathBuf;

/// `start` subcommand
///
#[derive(Command, Debug, Options)]
pub struct StartCmd {
    /// Path to configuration file
    #[options(short = "c", long = "config", help = "path to light_node.toml")]
    pub config: Option<PathBuf>,

    /// RPC address to request headers and validators from.
    #[options(free)]
    rpc_addr: String,
}

impl Runnable for StartCmd {
    /// Start the application.
    fn run(&self) {
        if let Err(err) = abscissa_tokio::run(&APPLICATION, async {
            eprintln!("TODO");
            process::exit(1);
        }) {
            eprintln!("Error while running application: {}", err);
            process::exit(1);
        }
    }
}

impl config::Override<LightNodeConfig> for StartCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(
        &self,
        mut config: LightNodeConfig,
    ) -> Result<LightNodeConfig, FrameworkError> {
        Ok(config)
    }
}
