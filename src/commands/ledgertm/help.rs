use abscissa::{Callable, Command};

use super::LedgertmCommand;

/// The `ledgertm help` subcommand
#[derive(Debug, Default, Options)]
pub struct HelpCommand {
    #[options(free)]
    pub args: Vec<String>,
}

impl Callable for HelpCommand {
    /// Print help for the `ledgertm` subcommand
    fn call(&self) {
        LedgertmCommand::print_usage(self.args.as_slice());
    }
}
