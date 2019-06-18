//! YubiHSM2 key management commands

mod export;
mod generate;
mod import;
mod list;

use self::{
    export::ExportCommand, generate::GenerateCommand, import::ImportCommand, list::ListCommand,
};
use abscissa::{Command, Help, Runnable};

/// Default key type to generate
pub const DEFAULT_KEY_TYPE: &str = "ed25519";

/// Default YubiHSM2 domain (internal partitioning)
pub const DEFAULT_DOMAINS: yubihsm::Domain = yubihsm::Domain::DOM1;

/// Default YubiHSM2 permissions for generated keys
pub const DEFAULT_CAPABILITIES: yubihsm::Capability = yubihsm::Capability::SIGN_EDDSA;

/// Default wrap key to use when exporting
pub const DEFAULT_WRAP_KEY: yubihsm::object::Id = 1;

/// The `yubihsm keys` subcommand
#[derive(Command, Debug, Options, Runnable)]
pub enum KeysCommand {
    #[options(help = "export an encrypted backup of a signing key inside the HSM device")]
    Export(ExportCommand),

    #[options(help = "generate an Ed25519 signing key inside the HSM device")]
    Generate(GenerateCommand),

    #[options(help = "show help for the 'yubihsm keys' subcommand")]
    Help(Help<Self>),

    #[options(help = "import validator signing key for the 'yubihsm keys' subcommand")]
    Import(ImportCommand),

    #[options(help = "list all suitable Ed25519 keys in the HSM")]
    List(ListCommand),
}

impl KeysCommand {
    /// Optional path to the configuration file
    pub(super) fn config_path(&self) -> Option<&String> {
        match self {
            KeysCommand::Export(export) => export.config.as_ref(),
            KeysCommand::Generate(generate) => generate.config.as_ref(),
            KeysCommand::List(list) => list.config.as_ref(),
            KeysCommand::Import(import) => import.config.as_ref(),
            _ => None,
        }
    }
}
