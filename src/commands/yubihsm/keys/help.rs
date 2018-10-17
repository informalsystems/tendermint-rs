use abscissa::{Callable, Command};

use super::KeysCommand;

/// The `yubihsm keys help` subcommand
#[derive(Debug, Default, Options)]
pub struct HelpCommand {
    #[options(free)]
    pub args: Vec<String>,
}

impl Callable for HelpCommand {
    /// Print help for the `yubihsm` subcommand
    fn call(&self) {
        KeysCommand::print_usage(self.args.as_slice());
    }
}
