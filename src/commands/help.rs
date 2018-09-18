//! The `help` subcommand

use abscissa::Command;

use super::KMSCommand;

/// The `help` subcommand
#[derive(Debug, Default, Options)]
pub struct HelpCommand {
    #[options(free)]
    pub args: Vec<String>,
}

impl HelpCommand {
    /// Print help message
    pub fn call(&self) -> ! {
        KMSCommand::print_usage(self.args.as_slice())
    }
}
