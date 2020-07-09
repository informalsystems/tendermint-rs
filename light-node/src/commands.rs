//! LightNode Subcommands
//!
//! The light client supports the following subcommands:
//! - `initialize`: subjectively initializes the light node with a given height and hash
//! - `start`: launches the light client
//! - `version`: print application version
//!
//! See the `impl Configurable` below for how to specify the path to the
//! application's configuration file.

mod initialize;
mod start;
mod version;

use self::{start::StartCmd, version::VersionCmd};
use crate::config::LightNodeConfig;
use abscissa_core::{
    config::Override, Command, Configurable, FrameworkError, Help, Options, Runnable,
};
use std::path::PathBuf;
use crate::commands::initialize::InitCmd;

/// LightNode Configuration Filename
pub const CONFIG_FILE: &str = "light_node.toml";

/// LightNode Subcommands
#[derive(Command, Debug, Options, Runnable)]
pub enum LightNodeCmd {
    /// The `help` subcommand
    #[options(help = "get usage information")]
    Help(Help<Self>),

    /// `intialize` the light node
    #[options(help = "subjectively initialize the light client with given subjective height and validator set hash")]
    Initialize(InitCmd),

    /// `start` the light node
    #[options(help = "start the light node daemon with the given config or command line params")]
    Start(StartCmd),

    /// `version` of the light node
    #[options(help = "display version information")]
    Version(VersionCmd),
}

/// This trait allows you to define how application configuration is loaded.
impl Configurable<LightNodeConfig> for LightNodeCmd {
    /// Location of the configuration file
    fn config_path(&self) -> Option<PathBuf> {
        // Check if the config file exists, and if it does not, ignore it.
        // If you'd like for a missing configuration file to be a hard error
        // instead, always return `Some(CONFIG_FILE)` here.
        let filename = PathBuf::from(CONFIG_FILE);

        if filename.exists() {
            Some(filename)
        } else {
            None
        }
    }

    /// Apply changes to the config after it's been loaded, e.g. overriding
    /// values in a config file using command-line options.
    ///
    /// This can be safely deleted if you don't want to override config
    /// settings from command-line options.
    fn process_config(&self, config: LightNodeConfig) -> Result<LightNodeConfig, FrameworkError> {
        match self {
            LightNodeCmd::Start(cmd) => cmd.override_config(config),
            _ => Ok(config),
        }
    }
}
