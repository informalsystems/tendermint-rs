//! Subcommands of the `tmkms` command-line application

mod keygen;
mod start;
mod version;
#[cfg(feature = "yubihsm")]
mod yubihsm;
#[cfg(feature = "yubihsm")]
pub use self::yubihsm::YubihsmCommand;
#[cfg(feature = "ledgertm")]
mod ledger;
#[cfg(feature = "ledgertm")]
pub use self::ledger::LedgerCommand;

pub use self::{keygen::KeygenCommand, start::StartCommand, version::VersionCommand};
use crate::config::{KmsConfig, CONFIG_ENV_VAR, CONFIG_FILE_NAME};
use abscissa::{Command, Configurable, Help, Runnable};
use std::{env, path::PathBuf};

/// Subcommands of the KMS command-line application
#[derive(Command, Debug, Options, Runnable)]
pub enum KmsCommand {
    /// `help` subcommand
    #[options(help = "show help for a command")]
    Help(Help<Self>),

    /// `keygen` subcommand
    #[options(help = "generate a new software signing key")]
    Keygen(KeygenCommand),

    /// `start` subcommand
    #[options(help = "start the KMS application")]
    Start(StartCommand),

    /// `version` subcommand
    #[options(help = "display version information")]
    Version(VersionCommand),

    /// `yubihsm` subcommand
    #[cfg(feature = "yubihsm")]
    #[options(help = "subcommands for YubiHSM2")]
    Yubihsm(YubihsmCommand),

    /// `ledgertm` subcommand
    #[cfg(feature = "ledgertm")]
    #[options(help = "subcommands for Ledger")]
    Ledger(LedgerCommand),
}

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

impl Configurable<KmsConfig> for KmsCommand {
    /// Get the path to the configuration file, either from selected subcommand
    /// or the default
    fn config_path(&self) -> Option<PathBuf> {
        let config = match self {
            KmsCommand::Start(run) => run.config.as_ref(),
            #[cfg(feature = "yubihsm")]
            KmsCommand::Yubihsm(yubihsm) => yubihsm.config_path(),
            #[cfg(feature = "ledgertm")]
            KmsCommand::Ledger(ledger) => ledger.config_path(),
            _ => return None,
        };

        let path = PathBuf::from(
            config
                .cloned()
                .or_else(|| env::var(CONFIG_ENV_VAR).ok())
                .unwrap_or_else(|| CONFIG_FILE_NAME.to_owned()),
        );

        Some(path)
    }
}
