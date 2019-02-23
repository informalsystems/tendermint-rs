use super::YubihsmCommand;
use abscissa::{Callable, Command};

/// The `yubihsm help` subcommand
#[derive(Debug, Default, Options)]
pub struct HelpCommand {
    #[options(free)]
    pub args: Vec<String>,
}

impl Callable for HelpCommand {
    /// Print help for the `yubihsm` subcommand
    fn call(&self) {
        YubihsmCommand::print_usage(self.args.as_slice());
    }
}
