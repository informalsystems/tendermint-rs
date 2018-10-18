mod generate;
mod help;
mod list;

use abscissa::Callable;

use self::{generate::GenerateCommand, help::HelpCommand, list::ListCommand};

/// The `yubihsm keys` subcommand
#[derive(Debug, Options)]
pub enum KeysCommand {
    #[options(help = "generate an Ed25519 signing key inside the HSM device")]
    Generate(GenerateCommand),

    #[options(help = "show help for the 'yubihsm keys' subcommand")]
    Help(HelpCommand),

    #[options(help = "list all suitable Ed25519 keys in the HSM")]
    List(ListCommand),
}

impl KeysCommand {
    /// Optional path to the configuration file
    pub(super) fn config_path(&self) -> Option<&str> {
        match self {
            KeysCommand::Generate(generate) => generate.config.as_ref().map(|s| s.as_ref()),
            KeysCommand::List(list) => list.config.as_ref().map(|s| s.as_ref()),
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
            KeysCommand::List(list) => list.call(),
        }
    }
}

// TODO: custom derive in abscissa
impl_command!(KeysCommand);
