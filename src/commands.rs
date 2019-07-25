//! Subcommands of the `tmkms` command-line application

#[cfg(feature = "ledgertm")]
mod ledger;
#[cfg(feature = "softsign")]
mod softsign;
mod start;
mod version;
#[cfg(feature = "yubihsm")]
mod yubihsm;

#[cfg(feature = "ledgertm")]
pub use self::ledger::LedgerCommand;
#[cfg(feature = "softsign")]
pub use self::softsign::SoftsignCommand;
#[cfg(feature = "yubihsm")]
pub use self::yubihsm::YubihsmCommand;

pub use self::{start::StartCommand, version::VersionCommand};
use crate::config::{KmsConfig, CONFIG_ENV_VAR, CONFIG_FILE_NAME};
use abscissa_core::{Command, Configurable, Help, Runnable};
use std::{env, path::PathBuf};

/// Subcommands of the KMS command-line application
#[derive(Command, Debug, Options, Runnable)]
pub enum KmsCommand {
    /// `help` subcommand
    #[options(help = "show help for a command")]
    Help(Help<Self>),

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

    /// `softsign` subcommand
    #[cfg(feature = "softsign")]
    #[options(help = "subcommands for software signer")]
    Softsign(SoftsignCommand),
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
            KmsCommand::Start(start) => start.config.as_ref(),
            #[cfg(feature = "yubihsm")]
            KmsCommand::Yubihsm(yubihsm) => yubihsm.config_path(),
            #[cfg(feature = "ledgertm")]
            KmsCommand::Ledger(ledger) => ledger.config_path(),
            _ => return None,
        };

        let path = config
            .cloned()
            .or_else(|| env::var(CONFIG_ENV_VAR).ok().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from(CONFIG_FILE_NAME));

        Some(path)
    }
}
