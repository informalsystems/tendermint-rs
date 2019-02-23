//! Subcommands of the `tmkms` command-line application

mod help;
mod keygen;
mod start;
mod version;
#[cfg(feature = "yubihsm")]
mod yubihsm;
#[cfg(feature = "yubihsm")]
pub use self::yubihsm::YubihsmCommand;

pub use self::{
    help::HelpCommand, keygen::KeygenCommand, start::StartCommand, version::VersionCommand,
};
use crate::config::{KmsConfig, CONFIG_FILE_NAME};
use abscissa::{Callable, LoadConfig};
use std::path::PathBuf;

/// Subcommands of the KMS command-line application
#[derive(Debug, Options)]
pub enum KmsCommand {
    #[options(help = "show help for a command")]
    Help(HelpCommand),

    #[options(help = "generate a new software signing key")]
    Keygen(KeygenCommand),

    #[options(help = "start the KMS application")]
    Start(StartCommand),

    #[options(help = "display version information")]
    Version(VersionCommand),

    #[cfg(feature = "yubihsm")]
    #[options(help = "subcommands for YubiHSM2")]
    Yubihsm(YubihsmCommand),
}

// TODO: refactor abscissa internally so this is all part of the proc macro
impl_command!(KmsCommand);

impl KmsCommand {
    /// Are we configured for verbose logging?
    pub fn verbose(&self) -> bool {
        match self {
            KmsCommand::Start(run) => run.verbose,
            #[cfg(feature = "yubihsm")]
            KmsCommand::Yubihsm(yubihsm) => yubihsm.verbose(),
            _ => false,
        }
    }
}

impl LoadConfig<KmsConfig> for KmsCommand {
    /// Get the path to the configuration file, either from selected subcommand
    /// or the default
    fn config_path(&self) -> Option<PathBuf> {
        let config = match self {
            KmsCommand::Start(run) => run.config.as_ref().map(|s| s.as_ref()),
            #[cfg(feature = "yubihsm")]
            KmsCommand::Yubihsm(yubihsm) => yubihsm.config_path(),
            _ => return None,
        };

        Some(PathBuf::from(config.unwrap_or(CONFIG_FILE_NAME)))
    }
}

// TODO: refactor abscissa internally so this is all part of the proc macro
impl Callable for KmsCommand {
    /// Call the given command chosen via the CLI
    fn call(&self) {
        match self {
            KmsCommand::Help(help) => help.call(),
            KmsCommand::Keygen(keygen) => keygen.call(),
            KmsCommand::Start(run) => run.call(),
            KmsCommand::Version(version) => version.call(),
            #[cfg(feature = "yubihsm")]
            KmsCommand::Yubihsm(yubihsm) => yubihsm.call(),
        }
    }
}
