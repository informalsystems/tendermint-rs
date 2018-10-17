//! The `help` subcommand

use abscissa::{Callable, Command};

use super::KmsCommand;

/// The `help` subcommand
#[derive(Debug, Default, Options)]
pub struct HelpCommand {
    #[options(free)]
    pub args: Vec<String>,
}

impl Callable for HelpCommand {
    /// Print help message
    fn call(&self) {
        KmsCommand::print_usage(self.args.as_slice());
    }
}
