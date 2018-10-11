//! The KMS `yubihsm` subcommand

use abscissa::Callable;

mod detect;
mod help;
mod keys;

pub use self::{detect::DetectCommand, help::HelpCommand, keys::KeysCommand};

/// The `yubihsm` subcommand
#[derive(Debug, Options)]
pub enum YubihsmCommand {
    #[options(help = "detect all YubiHSM2 devices connected via USB")]
    Detect(DetectCommand),

    #[options(help = "show help for the 'yubihsm' subcommand")]
    Help(HelpCommand),

    #[options(help = "key management subcommands")]
    Keys(KeysCommand),
}

// TODO: custom derive in abscissa
impl_command!(YubihsmCommand);

// TODO: refactor abscissa internally so this is all part of the proc macro
impl Callable for YubihsmCommand {
    /// Call the given command chosen via the CLI
    fn call(&self) {
        match self {
            YubihsmCommand::Detect(detect) => detect.call(),
            YubihsmCommand::Help(help) => help.call(),
            YubihsmCommand::Keys(keys) => keys.call(),
        }
    }
}

impl YubihsmCommand {
    pub(super) fn config_path(&self) -> Option<&str> {
        match self {
            YubihsmCommand::Detect(detect) => detect.config.as_ref().map(|s| s.as_ref()),
            YubihsmCommand::Keys(keys) => keys.config_path(),
            _ => None,
        }
    }
}
