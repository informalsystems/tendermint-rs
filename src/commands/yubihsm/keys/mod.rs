mod generate;
mod help;
mod import;
mod list;

use self::{
    generate::GenerateCommand, help::HelpCommand, import::ImportCommand, list::ListCommand,
};
use abscissa::Callable;

/// Default key type to generate
pub const DEFAULT_KEY_TYPE: &str = "ed25519";

/// Default YubiHSM2 domain (internal partitioning)
pub const DEFAULT_DOMAINS: yubihsm::Domain = yubihsm::Domain::DOM1;

/// Default YubiHSM2 permissions for generated keys
pub const DEFAULT_CAPABILITIES: yubihsm::Capability = yubihsm::Capability::SIGN_EDDSA;

/// The `yubihsm keys` subcommand
#[derive(Debug, Options)]
pub enum KeysCommand {
    #[options(help = "generate an Ed25519 signing key inside the HSM device")]
    Generate(GenerateCommand),

    #[options(help = "show help for the 'yubihsm keys' subcommand")]
    Help(HelpCommand),

    #[options(help = "import validator signing key for the 'yubihsm keys' subcommand")]
    Import(ImportCommand),

    #[options(help = "list all suitable Ed25519 keys in the HSM")]
    List(ListCommand),
}

impl KeysCommand {
    /// Optional path to the configuration file
    pub(super) fn config_path(&self) -> Option<&str> {
        match self {
            KeysCommand::Generate(generate) => generate.config.as_ref().map(|s| s.as_ref()),
            KeysCommand::List(list) => list.config.as_ref().map(|s| s.as_ref()),
            KeysCommand::Import(import) => import.config.as_ref().map(|s| s.as_ref()),
            _ => None,
        }
    }
}

// TODO: refactor abscissa internally so this is all part of the proc macro
impl Callable for KeysCommand {
    /// Call the given command chosen via the CLI
    fn call(&self) {
        match self {
            KeysCommand::Generate(generate) => generate.call(),
            KeysCommand::Help(help) => help.call(),
            KeysCommand::Import(import) => import.call(),
            KeysCommand::List(list) => list.call(),
        }
    }
}

// TODO: custom derive in abscissa
impl_command!(KeysCommand);
