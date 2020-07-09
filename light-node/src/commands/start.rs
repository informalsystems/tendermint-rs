//! `start` subcommand - start the light node.

/// App-local prelude includes `app_reader()`/`app_writer()`/`app_config()`
/// accessors along with logging macros. Customize as you see fit.
use abscissa_core::{config, Command, FrameworkError, Options, Runnable};
use std::process;

use crate::application::{app_config, APPLICATION};
use crate::config::LightNodeConfig;

use abscissa_core::path::PathBuf;
use std::net::SocketAddr;
use tendermint_light_client::store::sled::SledStore;
use tendermint_light_client::store::LightStore;
use tendermint_light_client::types::Status;

/// `start` subcommand
///
#[derive(Command, Debug, Options)]
pub struct StartCmd {
    /// Path to configuration file
    #[options(
        short = "l",
        long = "listen",
        help = "address the rpc server will serve"
    )]
    pub listen_addr: Option<SocketAddr>,

    /// Path to configuration file
    #[options(short = "c", long = "config", help = "path to light_node.toml")]
    pub config: Option<PathBuf>,
}

impl Runnable for StartCmd {
    /// Start the application.
    fn run(&self) {
        if let Err(err) = abscissa_tokio::run(&APPLICATION, async {
            // TODO: handle errors properly:
            let primary_db_path = app_config().light_clients.first().unwrap().db_path.clone();
            let db = sled::open(primary_db_path).unwrap_or_else(|e| {
                println!("[error] could not open database: {}", e);
                std::process::exit(1);
            });

            let primary_store = SledStore::new(db);

            if primary_store.latest(Status::Verified).is_none() {
                println!("[error] no trusted state in store for primary, please initialize with the `initialize` subcommand first");
                std::process::exit(1);
            }
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
        // Todo figure out if other options would be reasonable to overwrite via CLI arguments.
        if let Some(addr) = self.listen_addr {
            config.rpc_config.listen_addr = addr;
        }
        Ok(config)
    }
}
