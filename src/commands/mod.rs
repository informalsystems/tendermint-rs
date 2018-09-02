//! Subcommands of the `cosmos-kms` command-line application

use abscissa::{Callable, LoadConfig};
use std::path::PathBuf;

mod help;
mod keygen;
mod run;
mod version;

pub use self::{
    help::HelpCommand, keygen::KeygenCommand, run::RunCommand, version::VersionCommand,
};
use config::{KMSConfig, CONFIG_FILE_NAME};

/// Subcommands of the KMS command-line application
#[derive(Debug, Options)]
pub enum KMSCommand {
    #[options(help = "show help for a command")]
    Help(HelpCommand),

    #[cfg(feature = "dalek-provider")]
    #[options(help = "generate a new signing key")]
    Keygen(KeygenCommand),

    #[options(help = "run the KMS application")]
    Run(RunCommand),

    #[options(help = "display version information")]
    Version(VersionCommand),
}

// TODO: refactor abscissa internally so this is all part of the proc macro
impl_command!(KMSCommand);

impl KMSCommand {
    /// Are we configured for verbose logging?
    pub fn verbose(&self) -> bool {
        match self {
            KMSCommand::Run(run) => run.verbose,
            _ => false,
        }
    }
}

impl LoadConfig<KMSConfig> for KMSCommand {
    /// Get the path to the configuration file, either from selected subcommand
    /// or the default
    fn config_path(&self) -> Option<PathBuf> {
        match self {
            KMSCommand::Run(run) => Some(PathBuf::from(
                run.config
                    .as_ref()
                    .map(|s| s.as_ref())
                    .unwrap_or(CONFIG_FILE_NAME),
            )),
            _ => None,
        }
    }
}

// TODO: refactor abscissa internally so this is all part of the proc macro
impl Callable for KMSCommand {
    /// Call the given command chosen via the CLI
    fn call(&self) {
        match self {
            KMSCommand::Help(help) => help.call(),
            #[cfg(feature = "dalek-provider")]
            KMSCommand::Keygen(keygen) => keygen.call(),
            KMSCommand::Run(run) => run.call(),
            KMSCommand::Version(version) => version.call(),
        }
    }
}
