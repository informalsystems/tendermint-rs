//! The KMS `ledgertm` subcommand

use abscissa::Callable;

mod detect;
mod help;

pub use self::{detect::DetectCommand, help::HelpCommand};

/// The `ledgertm` subcommand
#[derive(Debug, Options)]
pub enum LedgertmCommand {
    #[options(help = "detect connected Ledger devices running the Tendermint app")]
    Detect(DetectCommand),

    #[options(help = "show help for the 'ledgertm' subcommand")]
    Help(HelpCommand),
}

impl_command!(LedgertmCommand);

impl Callable for LedgertmCommand {
    /// Call the given command chosen via the CLI
    fn call(&self) {
        match self {
            LedgertmCommand::Detect(detect) => detect.call(),
            LedgertmCommand::Help(help) => help.call(),
        }
    }
}

impl LedgertmCommand {}
